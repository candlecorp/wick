use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use vino_codec::messagepack::serialize;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};
use vino_transport::message_transport::stream::BoxedTransportStream;
use vino_transport::TransportMap;
pub use wapc::WasiParams;

use crate::error::LinkError;
use crate::wapc_module::WapcModule;
use crate::wasm_host::{
  WasmHost,
  WasmHostBuilder,
};
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  host: Arc<WasmHost>,
  #[allow(unused)]
  config: Vec<u8>,
}

pub type HostLinkCallback =
  dyn Fn(&str, &str, TransportMap) -> Result<Vec<TransportWrapper>, LinkError> + Sync + Send;

impl Provider {
  pub fn try_load(
    module: &WapcModule,
    config: Option<HashMap<String, String>>,
    wasi_options: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
  ) -> Result<Self, Error> {
    let mut builder = WasmHostBuilder::new();
    if let Some(opts) = wasi_options {
      builder = builder.wasi_params(opts);
    }
    if let Some(callback) = callback {
      builder = builder.link_callback(callback);
    }
    let host = builder.build(module)?;

    let host = Arc::new(host);

    Ok(Self {
      host,
      config: serialize(&config.unwrap_or_default())?,
    })
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    trace!("WASM:INVOKE:[{}]", entity);
    let component = entity.name();
    let messagepack_map = payload
      .try_into_messagepack_bytes()
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    let payload =
      serialize(&messagepack_map).map_err(|e| RpcError::ProviderError(e.to_string()))?;

    let outputs = self.host.call(&component, &payload)?;

    Ok(Box::pin(outputs))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = self.host.get_components();

    trace!(
      "WASM:COMPONENTS:[{}]",
      signature
        .components
        .inner()
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(",")
    );

    Ok(vec![HostedType::Provider(signature.clone())])
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use maplit::hashmap;
  use tokio_stream::StreamExt;
  use vino_provider::native::prelude::*;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)
    .await?;

    let provider = Provider::try_load(
      &component,
      None,
      None,
      Some(Box::new(|_origin, _component, _payload| Ok(vec![]))),
    )?;
    let input = "Hello world";

    let job_payload = TransportMap::from_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(input),
    });

    let mut outputs = provider
      .invoke(Entity::component_direct("validate"), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.try_into()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
