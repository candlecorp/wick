use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
use tracing::debug;
use wick_packet::{Invocation, Packet, PacketStream};

use crate::error::RpcClientError;
use crate::rpc::invocation_service_client::InvocationServiceClient;
use crate::rpc::{InvocationRequest, ListRequest, StatsRequest, StatsResponse};
use crate::{convert_tonic_streaming, generated};

/// Create an RPC client form common configuration
pub async fn make_rpc_client<T: TryInto<Uri> + Send>(
  address: T,
  pem: Option<PathBuf>,
  key: Option<PathBuf>,
  ca: Option<PathBuf>,
  domain: Option<String>,
) -> Result<RpcClient, RpcClientError> {
  let uri: Uri = address
    .try_into()
    .map_err(|_| RpcClientError::Other("Could not parse URI".to_owned()))?;

  let mut builder = Channel::builder(uri);

  if let (Some(pem), Some(key)) = (pem, key) {
    let server_pem = tokio::fs::read(pem).await?;
    let server_key = tokio::fs::read(key).await?;
    let identity = Identity::from_pem(server_pem, server_key);

    let mut tls = ClientTlsConfig::new().identity(identity);

    if let Some(ca) = ca {
      debug!("using CA from {}", ca.to_string_lossy());
      let ca_pem = tokio::fs::read(ca).await?;
      let ca = Certificate::from_pem(ca_pem);
      tls = tls.ca_certificate(ca);
    }
    if let Some(domain) = domain {
      tls = tls.domain_name(domain);
    }

    builder = builder.tls_config(tls).map_err(RpcClientError::TlsError)?;
  } else if let Some(ca) = ca {
    debug!("using CA from {}", ca.to_string_lossy());

    let ca_pem = tokio::fs::read(ca).await?;
    let ca = Certificate::from_pem(ca_pem);
    let mut tls = ClientTlsConfig::new().ca_certificate(ca);
    if let Some(domain) = domain {
      tls = tls.domain_name(domain);
    }
    builder = builder.tls_config(tls).map_err(RpcClientError::TlsError)?;
  };

  let result = builder
    .timeout(Duration::from_secs(5))
    .rate_limit(5, Duration::from_secs(1))
    .concurrency_limit(256)
    .connect()
    .await;

  let channel = result.map_err(|e| {
    e.source().map_or(RpcClientError::UnspecifiedConnectionError, |e| {
      RpcClientError::ConnectionError(e.to_string())
    })
  })?;

  Ok(RpcClient::from_channel(InvocationServiceClient::new(channel)))
}

#[derive(Debug, Clone)]
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
  pub const fn from_channel(channel: InvocationServiceClient<Channel>) -> Self {
    Self { inner: channel }
  }

  /// Make a request to the stats RPC method
  pub async fn stats(&mut self, request: StatsRequest) -> Result<StatsResponse, RpcClientError> {
    debug!("making stats request");
    let result = self
      .inner
      .stats(request)
      .await
      .map_err(RpcClientError::StatsCallFailed)?;
    debug!("stats result: {:?}", result);
    Ok(result.into_inner())
  }

  /// Make a request to the list RPC method
  pub async fn list(&mut self) -> Result<Vec<wick_interface_types::ComponentSignature>, RpcClientError> {
    let request = ListRequest {};
    debug!("making list request");
    let result = self.inner.list(request).await.map_err(RpcClientError::ListCallFailed)?;
    debug!("list result: {:?}", result);
    let response = result.into_inner();

    response
      .components
      .into_iter()
      .map(wick_interface_types::ComponentSignature::try_from)
      .collect::<Result<_, _>>()
      .map_err(|e| RpcClientError::TypeConversion(e.to_string()))
  }

  /// Send an invoke RPC command with a raw RPC [Invocation] object.
  pub async fn invoke_raw(
    &mut self,
    request: impl Stream<Item = InvocationRequest> + Send + Sync + 'static,
  ) -> Result<PacketStream, RpcClientError> {
    debug!("making invocation ");
    let result = self
      .inner
      .invoke(request)
      .await
      .map_err(RpcClientError::InvocationFailed)?;
    debug!("invocation result: {:?}", result);

    // Need to do this because tonic::decode::Decoder is not Sync and can't be turned into a PacketStream.
    let stream = convert_tonic_streaming(result.into_inner());

    Ok(stream)
  }

  /// Send an invoke RPC command with an [Invocation] object.
  pub async fn invoke(&mut self, invocation: Invocation) -> Result<PacketStream, RpcClientError> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let (invocation, mut stream) = invocation.split();
    tx.send(InvocationRequest {
      data: Some(generated::wick::invocation_request::Data::Invocation(invocation.into())),
    })
    .map_err(|_e| RpcClientError::UnspecifiedConnectionError)?;
    tokio::spawn(async move {
      while let Some(packet) = stream.next().await {
        let packet = packet.map_or_else(|e| Packet::component_error(e.to_string()), |p| p);
        tx.send(InvocationRequest {
          data: Some(generated::wick::invocation_request::Data::Packet(packet.into())),
        })
        .map_err(|_e| RpcClientError::UnspecifiedConnectionError)?;
      }
      Ok::<_, RpcClientError>(())
    });

    self
      .invoke_raw(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
      .await
  }
}
