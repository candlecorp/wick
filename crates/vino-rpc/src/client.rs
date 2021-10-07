use log::debug;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use vino_entity::Entity;
use vino_transport::{
  MessageTransport,
  TransportMap,
  TransportStream,
  TransportWrapper,
};

use crate::rpc::invocation_service_client::InvocationServiceClient;
use crate::rpc::{
  Invocation,
  ListRequest,
  ListResponse,
  StatsRequest,
  StatsResponse,
};
use crate::types::conversions::convert_transport_map;

/// The error type that [RpcClient] methods produce.
#[derive(thiserror::Error, Debug)]
pub enum RpcClientError {
  /// An upstream error from [tonic].
  #[error("Upstream error: {0}")]
  RpcStatus(#[from] tonic::Status),
  /// An error related to [vino_transport].
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  /// Connection failed
  #[error("Connection failed: {0}")]
  ConnectionFailed(String),
}

#[derive(Debug)]
/// [RpcClient] wraps an [InvocationServiceClient] into a more usable package.
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

  /// Make a request to the stats RPC method
  pub async fn stats(&mut self, request: StatsRequest) -> Result<StatsResponse, RpcClientError> {
    debug!("Making stats request");
    let result = self.inner.stats(request).await?;
    debug!("Stats result: {:?}", result);
    Ok(result.into_inner())
  }

  /// Make a request to the list RPC method
  pub async fn list(&mut self, request: ListRequest) -> Result<ListResponse, RpcClientError> {
    debug!("Making list request");
    let result = self.inner.list(request).await?;
    debug!("List result: {:?}", result);
    Ok(result.into_inner())
  }

  /// Send an invoke RPC command with a raw RPC [Invocation] object.
  pub async fn invoke_raw(
    &mut self,
    request: crate::rpc::Invocation,
  ) -> Result<TransportStream, RpcClientError> {
    debug!("Making list request");
    let result = self.inner.invoke(request).await?;
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
