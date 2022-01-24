use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use vino_entity::Entity;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};
use vino_transport::TransportMap;
use vino_transport::{BoxedTransportStream, TransportWrapper};
use vino_types::*;
pub use wapc::WasiParams;

use crate::error::LinkError;
use crate::wapc_module::WapcModule;
use crate::wasi::config_to_wasi;
use crate::wasm_host::{WasmHost, WasmHostBuilder};
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Provider {
  pool: Arc<WasmHost>,
}

pub type HostLinkCallback =
  dyn Fn(&str, &str, TransportMap) -> Result<Vec<TransportWrapper>, LinkError> + Sync + Send;

impl Provider {
  pub fn try_load(
    module: &WapcModule,
    max_threads: usize,
    config: Option<serde_json::Value>,
    wasi_params: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
  ) -> Result<Self, Error> {
    let mut builder = WasmHostBuilder::new();

    // TODO: progagate a name for better messages.
    let name = "unnamed";

    // If we're passed a "wasi" field in the config map...
    if let Some(config) = config {
      let wasi_cfg = config.get("wasi");
      if wasi_cfg.is_some() {
        // extract and merge the wasi config with the passed wasi params.
        let wasi = config_to_wasi(wasi_cfg.cloned(), wasi_params)?;
        debug!("WASM[{}]:WASI[enabled]:CFG[{:?}]", name, wasi);
        builder = builder.wasi_params(wasi);
      }
    } else if let Some(opts) = wasi_params {
      // if we were passed wasi params, use those.
      debug!("WASM[{}]:WASI[enabled]:CFG[{:?}]", name, opts);
      builder = builder.wasi_params(opts);
    } else {
      debug!("WASM[{}]:WASI[disabled]", name);
    }
    builder = builder.max_threads(max_threads);

    if let Some(callback) = callback {
      builder = builder.link_callback(callback);
    }
    let host = builder.build(module)?;

    Ok(Self {
      pool: Arc::new(host),
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

    let outputs = pool.call(&component, &messagepack_map).await?;

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
  use vino_transport::MessageTransport;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../../integration/test-wapc-component/build/test_component_s.wasm",
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
