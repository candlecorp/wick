use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use vino_codec::messagepack::serialize;
use vino_entity::Entity;
// use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};
use vino_transport::TransportMap;
use vino_transport::{BoxedTransportStream, TransportWrapper};
use vino_types::*;
pub use wapc::WasiParams;

use crate::error::LinkError;
use crate::host_pool::HostPool;
// use crate::host_pool::HostPool;
use crate::wapc_module::WapcModule;
use crate::wasm_host::WasmHostBuilder;
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive()]
pub struct Provider {
  pool: Arc<HostPool>,
  // pool: WasmHost,
  #[allow(unused)]
  config: Vec<u8>,
}

impl std::fmt::Debug for Provider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Provider")
      .field("config", &self.config)
      .finish()
  }
}

pub type HostLinkCallback =
  dyn Fn(&str, &str, TransportMap) -> Result<Vec<TransportWrapper>, LinkError> + Sync + Send;

impl Provider {
  pub fn try_load(
    module: &WapcModule,
    max_threads: usize,
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

    let pool = HostPool::start_hosts(move || Box::new(host.clone()), max_threads);

    Ok(Self {
      pool: Arc::new(pool),
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
    let pool = self.pool.clone();

    let outputs = pool.call(&component, &messagepack_map)?;

    Ok(Box::pin(outputs))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = self.pool.get_components();

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

    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use maplit::hashmap;
  use tokio_stream::StreamExt;
  use vino_transport::MessageTransport;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../integration/test-wapc-component/build/test_component_s.wasm",
    )?)
    .await?;

    let provider = Provider::try_load(
      &component,
      2,
      None,
      None,
      Some(Box::new(|_origin, _component, _payload| Ok(vec![]))),
    )?;
    let input = "Hello world";

    let job_payload = TransportMap::from_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(input),
    });
    debug!("payload: {:?}", job_payload);
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
