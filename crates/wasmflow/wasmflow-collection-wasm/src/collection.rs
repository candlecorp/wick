use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio_stream::StreamExt;
pub use wapc::WasiParams;
use wasmflow_codec::{json, messagepack};
use wasmflow_collection_link::CollectionLink;
use wasmflow_entity::Entity;
use wasmflow_invocation::Invocation;
use wasmflow_packet::{PacketMap, PacketWrapper};
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_sdk::types::HostedType;
use wasmflow_transport::{MessageTransport, Serialized, TransportMap, TransportStream};

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
pub struct Collection {
  pool: Arc<WasmHost>,
}

pub type HostLinkCallback = dyn Fn(&str, &str, PacketMap) -> Result<Vec<PacketWrapper>, LinkError> + Sync + Send;

impl Collection {
  pub fn try_load(
    module: &WapcModule,
    max_threads: usize,
    config: Option<serde_json::Value>,
    wasi_params: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
  ) -> Result<Self, Error> {
    let mut builder = WasmHostBuilder::new();

    let name = module.name().clone().unwrap_or_else(|| module.id().clone());

    // If we're passed a "wasi" field in the config map...
    if let Some(config) = config {
      let wasi_cfg = config.get("wasi");
      if wasi_cfg.is_some() {
        // extract and merge the wasi config with the passed wasi params.
        let wasi = config_to_wasi(wasi_cfg.cloned(), wasi_params)?;
        debug!(id=%name, config=?wasi, "wasi enabled");
        builder = builder.wasi_params(wasi);
      }
    } else if let Some(opts) = wasi_params {
      // if we were passed wasi params, use those.
      debug!(id=%name, config=?opts, "wasi enabled");

      builder = builder.wasi_params(opts);
    } else {
      debug!(id = %name, "wasi disabled");
    }
    builder = builder.max_threads(max_threads);

    if let Some(callback) = callback {
      builder = builder.link_callback(callback);
    }
    let host = builder.build(module)?;

    Ok(Self { pool: Arc::new(host) })
  }

  pub async fn exec_main(&self, origin: Entity, argv: Vec<String>) -> u32 {
    let mut transport_map = TransportMap::default();
    transport_map.insert("argv", MessageTransport::success(&argv));
    let target = Entity::component(origin.namespace(), "main");
    let link = CollectionLink::new(origin.clone(), target.clone());

    transport_map.insert("network", MessageTransport::success(&link));
    let invocation = Invocation::new(origin, target, transport_map, None);
    let result = self.invoke(invocation).await;
    if let Err(e) = result {
      error!("main() died with fatal error: {}", e);
      return 6;
    }
    let output = result.unwrap();
    let packets: Vec<_> = output.collect().await;
    for packet in packets {
      if packet.port == "code" {
        return if let MessageTransport::Failure(err) = packet.payload {
          error!("main() component returned error: {}", err.message());
          1
        } else {
          match packet.payload.deserialize::<u32>() {
            Ok(code) => code,
            Err(e) => {
              error!("Could not get code from main() component: {}", e);
              2
            }
          }
        };
      }
    }
    error!("No exit code received");
    3
  }
}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    trace!(target = %invocation.target, "wasm invoke");
    let component = invocation.target.name();
    let messagepack_map =
      try_into_messagepack_bytes(invocation.payload).map_err(|e| RpcError::CollectionError(e.to_string()))?;
    let pool = self.pool.clone();

    let config = invocation.config.map(|v| v.into_messagepack());
    let state = invocation.state.map(|v| v.into_messagepack());

    let outputs = pool.call(component, &messagepack_map, config, state).await?;

    Ok(outputs)
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

    Ok(vec![HostedType::Collection(signature.clone())])
  }
}

fn try_into_messagepack_bytes(payload: TransportMap) -> Result<HashMap<String, Vec<u8>>, Error> {
  let mut map = HashMap::new();
  for (k, v) in payload.into_inner() {
    let bytes = match v {
      MessageTransport::Success(success) => match success {
        Serialized::MessagePack(bytes) => bytes,
        Serialized::Struct(v) => messagepack::serialize(&v)?,
        Serialized::Json(v) => {
          let value: serde_value::Value = json::deserialize(&v)?;
          messagepack::serialize(&value)?
        }
      },
      MessageTransport::Failure(_) => {
        unreachable!("Failure packets must be handled by the interpreter, not passed to a component")
      }
      MessageTransport::Signal(_) => {
        unreachable!("Signal packets must be handled by the interpreter, not passed to a component")
      }
    };
    map.insert(k, bytes);
  }
  Ok(map)
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use wasmflow_entity::Entity;
  use wasmflow_packet::PacketMap;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../../integration/test-wasm-component/build/test_component_s.wasm",
    )?)
    .await?;

    let collection = Collection::try_load(
      &component,
      2,
      None,
      None,
      Some(Box::new(|_origin, _component, _payload| Ok(vec![]))),
    )?;
    let input = "Hello world";

    let job_payload = PacketMap::from([("input", input)]);
    debug!("payload: {:?}", job_payload);
    let entity = Entity::local("validate");
    let invocation = Invocation::new_test(file!(), entity, job_payload, None);
    let mut outputs = collection.invoke(invocation).await?;
    debug!("Invocation complete");
    let packets: Vec<_> = outputs.drain_port("output").await?;
    debug!("Output packets: {:?}", packets);
    let output = packets[0].clone();

    println!("payload from [{}]: {:?}", output.port, output.payload);
    let output: String = output.payload.deserialize()?;

    println!("output: {:?}", output);
    assert_eq!(output, input);
    Ok(())
  }
}
