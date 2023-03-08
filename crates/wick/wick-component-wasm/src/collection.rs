use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use wasmrs_host::WasiParams;
use wick_config::Permissions;
use wick_interface_types::HostedType;
use wick_packet::{Invocation, PacketStream};
use wick_rpc::{RpcHandler, RpcResult};

use crate::error::LinkError;
use crate::helpers::WickWasmModule;
use crate::wasm_host::{WasmHost, WasmHostBuilder};
use crate::Error;

pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Collection {
  pool: Arc<WasmHost>,
}

pub type HostLinkCallback =
  dyn Fn(&str, &str, PacketStream) -> BoxedFuture<Result<PacketStream, LinkError>> + Send + Sync;

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

impl Collection {
  pub fn try_load(
    module: &WickWasmModule,
    max_threads: usize,
    config: Option<Permissions>,
    additional_config: Option<Permissions>,
    callback: Option<Box<HostLinkCallback>>,
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

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> RpcResult<PacketStream> {
    trace!(target = %invocation.target, "wasm invoke");
    let component = invocation.target.name();

    let outputs = self.pool.call(component, stream, None)?;

    Ok(outputs)
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = self.pool.get_components();

    trace!(
      "WASM:COMPONENTS:[{}]",
      signature
        .operations
        .inner()
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(",")
    );

    Ok(vec![HostedType::Collection(signature.clone())])
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result as TestResult;
  use futures::StreamExt;
  use wick_packet::{packet_stream, packets, Entity, Packet};

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_component() -> TestResult<()> {
    let component = crate::helpers::load_wasm_from_file(&PathBuf::from_str(
      "../../integration/test-baseline-component/build/baseline.signed.wasm",
    )?)
    .await?;

    let collection = Collection::try_load(
      &component,
      2,
      None,
      None,
      Some(Box::new(|_origin, _component, _payload| {
        Box::pin(async { Ok(packet_stream!(("test", "test"))) })
      })),
    )?;

    let stream = packets!(("left", 10), ("right", 20));
    println!("{:#?}", stream);
    let entity = Entity::local("add");
    let invocation = Invocation::new(Entity::test(file!()), entity, None);
    let outputs = collection.invoke(invocation, stream.into()).await?;
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
