use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use tracing::Span;
use wasmrs_host::WasiParams;
use wick_config::config::components::Permissions;
use wick_packet::{Entity, Invocation, OperationConfig, PacketStream};
use wick_rpc::RpcHandler;

use crate::helpers::WickWasmModule;
use crate::wasm_host::{SetupPayload, WasmHost, WasmHostBuilder};
use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct WasmComponent {
  host: Arc<WasmHost>,
}

fn permissions_to_wasi_params(perms: &Permissions) -> WasiParams {
  let preopened_dirs = perms.dirs().values().cloned().collect();
  let map_dirs = perms.dirs().clone().into_iter().collect();
  WasiParams {
    map_dirs,
    preopened_dirs,
    ..Default::default()
  }
}

impl WasmComponent {
  pub async fn try_load(
    module: &WickWasmModule,
    max_threads: usize,
    config: Option<Permissions>,
    additional_config: Option<Permissions>,
    callback: Option<Arc<RuntimeCallback>>,
    provided: HashMap<String, String>,
    span: Span,
  ) -> Result<Self, Error> {
    let mut builder = WasmHostBuilder::new(span.clone());

    let name = module.name().clone().unwrap_or_else(|| module.id().clone());

    // If we're passed a "wasi" field in the config map...
    if let Some(config) = config {
      span.in_scope(|| debug!(id=%name, config=?config, "wasi enabled"));
      builder = builder.wasi_params(permissions_to_wasi_params(&config));
    } else if let Some(opts) = additional_config {
      // if we were passed wasi params, use those.
      span.in_scope(|| debug!(id=%name, config=?opts, "wasi enabled"));

      builder = builder.wasi_params(permissions_to_wasi_params(&opts));
    } else {
      span.in_scope(|| debug!(id = %name, "wasi disabled"));
    }
    builder = builder.max_threads(max_threads);

    if let Some(callback) = callback {
      builder = builder.link_callback(callback);
    }
    let host = builder.build(module)?;
    let setup = SetupPayload::new(&Entity::component(module.name().clone().unwrap_or_default()), provided);
    host.setup(setup).await?;

    Ok(Self { host: Arc::new(host) })
  }
}

impl Component for WasmComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<OperationConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, config=?data, "wasm invoke"));

    let outputs = self.host.call(invocation, stream, data);

    Box::pin(async move { outputs.map_err(ComponentError::new) })
  }

  fn list(&self) -> &wick_interface_types::ComponentSignature {
    self.host.get_operations()
  }
}

impl RpcHandler for WasmComponent {}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result;
  use flow_component::panic_callback;
  use futures::StreamExt;
  use serde_json::json;
  use wick_packet::{packet_stream, packets, Entity, Packet};

  use super::*;

  async fn load_component() -> Result<WasmComponent> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../../integration/test-baseline-component/build/baseline.signed.wasm",
    )?)
    .await?;

    let c = WasmComponent::try_load(
      &component,
      2,
      None,
      None,
      Some(Arc::new(|_, _, _, _, _, _| {
        Box::pin(async { Ok(packet_stream!(("test", "test"))) })
      })),
      Default::default(),
      Span::current(),
    )
    .await?;
    Ok(c)
  }

  #[test_logger::test(tokio::test)]
  #[ignore = "TODO: fix this from hanging. It works when run via the interpreter but not the test harness."]
  async fn test_component_error() -> Result<()> {
    let component = load_component().await?;
    let stream = packets!(("input", "10"));
    println!("{:#?}", stream);
    let invocation = Invocation::test(file!(), Entity::local("error"), None)?;
    let config = json!({});
    let outputs = component
      .handle(invocation, stream.into(), Some(config.try_into()?), panic_callback())
      .await?;
    debug!("Got stream");
    let mut packets: Vec<_> = outputs.collect().await;
    debug!("Output packets: {:?}", packets);

    let output = packets.pop().unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::component_error("Component sent invalid context"));
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_component_add() -> Result<()> {
    let component = load_component().await?;
    let stream = packets!(("left", 10), ("right", 20));
    println!("{:#?}", stream);
    let invocation = Invocation::test(file!(), Entity::local("add"), None)?;
    let config = json!({});
    let outputs = component
      .handle(invocation, stream.into(), Some(config.try_into()?), panic_callback())
      .await?;
    debug!("Got stream");
    let mut packets: Vec<_> = outputs.collect().await;
    debug!("Output packets: {:?}", packets);

    let _ = packets.pop();
    let output = packets.pop().unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", 30));
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_component_power() -> Result<()> {
    let component = load_component().await?;
    let stream = packets!(("input", 44));
    println!("{:#?}", stream);
    let invocation = Invocation::test(file!(), Entity::local("power"), None)?;
    let config = json!({
      "exponent": 2
    });
    let outputs = component
      .handle(invocation, stream.into(), Some(config.try_into()?), panic_callback())
      .await?;
    let mut packets: Vec<_> = outputs.collect().await;
    debug!("Output packets: {:?}", packets);

    let _ = packets.pop();
    let output = packets.pop().unwrap().unwrap();

    println!("output: {:?}", output);
    assert_eq!(output, Packet::encode("output", 1936));
    Ok(())
  }
}
