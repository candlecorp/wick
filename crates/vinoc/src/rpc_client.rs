use std::convert::TryInto;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use nkeys::KeyPair;
use tonic::transport::{
  Certificate,
  Channel,
  ClientTlsConfig,
  Identity,
  Uri,
};
use vino_entity::Entity;
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;
use vino_rpc::rpc::{
  ListRequest,
  ListResponse,
  StatsRequest,
  StatsResponse,
};
use vino_runtime::prelude::Invocation;
use vino_transport::TransportMap;

use crate::error::ControlError;

pub(crate) async fn rpc_client(
  address: Ipv4Addr,
  port: u16,
  pem: Option<PathBuf>,
  key: Option<PathBuf>,
  ca: Option<PathBuf>,
  domain: Option<String>,
) -> Result<RpcClient, ControlError> {
  let url = format!("https://{}:{}", address, port);
  let uri = Uri::from_str(&url)
    .map_err(|_| crate::Error::Other(format!("Could not create URI from: {}", url)))?;

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

  Ok(RpcClient::new(InvocationServiceClient::new(channel)))
}

#[derive(thiserror::Error, Debug)]
pub enum RpcClientError {
  #[error("Upstream error: {0}")]
  RpcStatus(#[from] tonic::Status),
  #[error("Error from runtime: {0}")]
  RuntimeError(String),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
}
impl From<vino_runtime::Error> for RpcClientError {
  fn from(e: vino_runtime::Error) -> Self {
    RpcClientError::RuntimeError(e.to_string())
  }
}

pub(crate) struct RpcClient {
  inner: InvocationServiceClient<Channel>,
}

impl RpcClient {
  pub(crate) fn new(client: InvocationServiceClient<Channel>) -> Self {
    Self { inner: client }
  }

  pub(crate) async fn stats(
    &mut self,
    request: StatsRequest,
  ) -> Result<StatsResponse, RpcClientError> {
    debug!("Making stats request");
    let result = self.inner.stats(request).await?;
    debug!("Stats result: {:?}", result);
    Ok(result.into_inner())
  }

  pub(crate) async fn list(
    &mut self,
    request: ListRequest,
  ) -> Result<ListResponse, RpcClientError> {
    debug!("Making list request");
    let result = self.inner.list(request).await?;
    debug!("List result: {:?}", result);
    Ok(result.into_inner())
  }

  pub(crate) async fn invoke_raw(
    &mut self,
    request: vino_rpc::rpc::Invocation,
  ) -> Result<tonic::Streaming<vino_rpc::rpc::Output>, RpcClientError> {
    debug!("Making list request");
    let result = self.inner.invoke(request).await?;
    debug!("Invocation result: {:?}", result);
    Ok(result.into_inner())
  }

  pub(crate) async fn invoke(
    &mut self,
    component: String,
    payload: TransportMap,
  ) -> Result<tonic::Streaming<vino_rpc::rpc::Output>, RpcClientError> {
    let kp = KeyPair::new_server();

    let rpc_invocation: vino_rpc::rpc::Invocation = Invocation::new(
      Entity::client(kp.public_key()),
      Entity::component_direct(component),
      payload,
    )
    .try_into()?;

    let stream = self.invoke_raw(rpc_invocation).await?;

    Ok(stream)
  }

  pub(crate) async fn invoke_from_json(
    &mut self,
    component: String,
    data: &str,
    transpose: bool,
  ) -> Result<tonic::Streaming<vino_rpc::rpc::Output>, RpcClientError> {
    let mut payload = TransportMap::from_json_str(data)?;
    if transpose {
      payload.transpose_output_name();
    }

    self.invoke(component, payload).await
  }
}
