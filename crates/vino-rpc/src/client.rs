use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use log::debug;
use tokio_stream::StreamExt;
use tonic::transport::{
  Certificate,
  Channel,
  ClientTlsConfig,
  Identity,
  Uri,
};
use vino_entity::Entity;
use vino_transport::{
  MessageTransport,
  TransportMap,
  TransportStream,
  TransportWrapper,
};

use crate::error::RpcClientError;
use crate::rpc::invocation_service_client::InvocationServiceClient;
use crate::rpc::{
  Invocation,
  ListRequest,
  ListResponse,
  StatsRequest,
  StatsResponse,
};
use crate::types::conversions::convert_transport_map;

/// Create an RPC client form common configuration
pub async fn make_rpc_client(
  address: Ipv4Addr,
  port: u16,
  pem: Option<PathBuf>,
  key: Option<PathBuf>,
  ca: Option<PathBuf>,
  domain: Option<String>,
) -> Result<RpcClient, RpcClientError> {
  let url = format!("https://{}:{}", address, port);
  let uri = Uri::from_str(&url)
    .map_err(|_| RpcClientError::Other(format!("Could not create URI from: {}", url)))?;

  let mut builder = Channel::builder(uri);

  if let (Some(pem), Some(key)) = (pem, key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);

    let mut tls = ClientTlsConfig::new().identity(identity);

    if let Some(ca) = ca {
      debug!("Using CA from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.ca_certificate(ca);
    }
    if let Some(domain) = domain {
      tls = tls.domain_name(domain);
    }
    builder = builder.tls_config(tls)?;
  } else if let Some(ca) = ca {
    debug!("Using CA from {}", ca.to_string_lossy());

    let ca_pem = tokio::fs::read(ca).await?;
    let ca = Certificate::from_pem(ca_pem);
    let mut tls = ClientTlsConfig::new().ca_certificate(ca);
    if let Some(domain) = domain {
      tls = tls.domain_name(domain);
    }
    builder = builder.tls_config(tls)?;
  };

  let channel = builder
    .timeout(Duration::from_secs(5))
    .rate_limit(5, Duration::from_secs(1))
    .concurrency_limit(256)
    .connect()
    .await?;

  Ok(RpcClient::from_channel(InvocationServiceClient::new(
    channel,
  )))
}

#[derive(Debug)]
/// [RpcClient] wraps an [InvocationServiceClient] into a more usable package.
#[must_use]
pub struct RpcClient {
  inner: InvocationServiceClient<Channel>,
}

impl RpcClient {
  /// Instantiate a new [RpcClient] connected to the passed URI.
  pub async fn new(uri: String) -> Result<Self, RpcClientError> {
    let client = InvocationServiceClient::connect(uri)
      .await
      .map_err(|e| RpcClientError::ConnectionFailed(e.to_string()))?;

    Ok(Self { inner: client })
  }

  /// Instantiate a new [RpcClient] from an existing InvocationServiceClient.
  pub fn from_channel(channel: InvocationServiceClient<Channel>) -> Self {
    Self { inner: channel }
  }

  /// Make a request to the stats RPC method
  pub async fn stats(&mut self, request: StatsRequest) -> Result<StatsResponse, RpcClientError> {
    debug!("Making stats request");
    let result = self
      .inner
      .stats(request)
      .await
      .map_err(RpcClientError::StatsCallFailed)?;
    debug!("Stats result: {:?}", result);
    Ok(result.into_inner())
  }

  /// Make a request to the list RPC method
  pub async fn list(&mut self, request: ListRequest) -> Result<ListResponse, RpcClientError> {
    debug!("Making list request");
    let result = self
      .inner
      .list(request)
      .await
      .map_err(RpcClientError::ListCallFailed)?;
    debug!("List result: {:?}", result);
    Ok(result.into_inner())
  }

  /// Send an invoke RPC command with a raw RPC [Invocation] object.
  pub async fn invoke_raw(
    &mut self,
    request: crate::rpc::Invocation,
  ) -> Result<TransportStream, RpcClientError> {
    debug!("Making invocation: {:?}", request);
    let result = self
      .inner
      .invoke(request)
      .await
      .map_err(RpcClientError::InvocationFailed)?;
    debug!("Invocation result: {:?}", result);
    let stream = result.into_inner();

    let mapped = stream.map::<TransportWrapper, _>(|o| -> TransportWrapper {
      match o {
        Ok(o) => o.into(),
        Err(e) => TransportWrapper::component_error(MessageTransport::error(format!(
          "Error converting RPC output to MessageTransports: {}",
          e
        ))),
      }
    });
    Ok(TransportStream::new(mapped))
  }

  /// Send an invoke RPC command
  pub async fn invoke(
    &mut self,
    origin: String,
    component: String,
    payload: TransportMap,
  ) -> Result<TransportStream, RpcClientError> {
    let rpc_invocation = Invocation {
      origin,
      target: Entity::component_direct(component).url(),
      msg: convert_transport_map(payload),
      id: "None".to_owned(),
    };

    let stream = self.invoke_raw(rpc_invocation).await?;

    Ok(stream)
  }

  /// Make an invocation with data passed as a JSON string.
  pub async fn invoke_from_json(
    &mut self,
    origin: String,
    component: String,
    data: &str,
    transpose: bool,
  ) -> Result<TransportStream, RpcClientError> {
    let mut payload = TransportMap::from_json_output(data)?;
    if transpose {
      payload.transpose_output_name();
    }

    self.invoke(origin, component, payload).await
  }
}
