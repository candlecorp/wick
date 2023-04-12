use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use wasmrs_host::WasiParams;
use wick_config::config::components::Permissions;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::RpcHandler;

use crate::helpers::WickWasmModule;
use crate::wasm_host::{WasmHost, WasmHostBuilder};
use crate::Error;

// pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
// pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct WasmComponent {
  pool: Arc<WasmHost>,
}

fn permissions_to_wasi_params(perms: Permissions) -> WasiParams {
  debug!(params=?perms, "Collection permissions");
  let preopened_dirs = perms.dirs.values().cloned().collect();
  let map_dirs = perms.dirs.into_iter().collect();
  let params = WasiParams {
    map_dirs,
    preopened_dirs,
    ..Default::default()
  };
  debug!(?params, "WASI configuration");
  params
}

impl WasmComponent {
  pub fn try_load(
    module: &WickWasmModule,
    max_threads: usize,
    config: Option<Permissions>,
    additional_config: Option<Permissions>,
    callback: Option<Arc<RuntimeCallback>>,
  ) -> Result<Self, Error> {
    let mut builder = WasmHostBuilder::new();

    let name = module.name().clone().unwrap_or_else(|| module.id().clone());

    // If we're passed a "wasi" field in the config map...
    if let Some(config) = config {
      debug!(id=%name, config=?config, "wasi enabled");
      builder = builder.wasi_params(permissions_to_wasi_params(config));
    } else if let Some(opts) = additional_config {
      // if we were passed wasi params, use those.
      debug!(id=%name, config=?opts, "wasi enabled");

      builder = builder.wasi_params(permissions_to_wasi_params(opts));
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
}

impl Component for WasmComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<serde_json::Value>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    trace!(target = %invocation.target, "wasm invoke");
    let component = invocation.target.name();

    let outputs = self.pool.call(component, stream, None);

    Box::pin(async move { outputs.map_err(ComponentError::new) })
  }

  fn list(&self) -> &wick_interface_types::ComponentSignature {
    let signature = self.pool.get_operations();

    trace!(
      "WASM:COMPONENTS:[{}]",
      signature
        .operations
        .iter()
        .map(|op| op.name.clone())
        .collect::<Vec<_>>()
        .join(",")
    );
    signature
  }
}

impl RpcHandler for WasmComponent {}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use flow_component::panic_callback;
  use futures::StreamExt;
  use wick_packet::{packet_stream, packets, Entity, Packet};

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../../integration/test-baseline-component/build/baseline.signed.wasm",
    )?)
    .await?;

    let collection = WasmComponent::try_load(
      &component,
      2,
      None,
      None,
      Some(Arc::new(|_, _, _, _| {
        Box::pin(async { Ok(packet_stream!(("test", "test"))) })
      })),
    )?;

    let stream = packets!(("left", 10), ("right", 20));
    println!("{:#?}", stream);
    let entity = Entity::local("add");
    let invocation = Invocation::new(Entity::test(file!()), entity, None);
    let outputs = collection
      .handle(invocation, stream.into(), None, panic_callback())
      .await?;
    debug!("Invocation complete");
    let mut packets: Vec<_> = outputs.collect().await;
    debug!("Output packets: {:?}", packets);

    let _ = packets.pop();
    let output = packets.pop().unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", 30));
    Ok(())
  }
}
