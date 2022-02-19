use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};
use vino_transport::Invocation;

use crate::components::Dispatcher;
use crate::error::NativeError;

#[derive(Clone, Debug, Default)]
pub struct Context {}

#[derive(Clone, Debug, Default)]
#[must_use]
pub struct Provider {
  context: Context,
}

impl From<NativeError> for Box<RpcError> {
  fn from(e: NativeError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

impl Provider {}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let result = Dispatcher::dispatch(invocation.target.name(), context, invocation.payload).await;
    let stream = result.map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = crate::components::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {}
