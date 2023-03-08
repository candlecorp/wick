pub(crate) mod error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use flow_graph_interpreter::{HandlerMap, NamespaceHandler};
use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_config::ComponentConfiguration;
use wick_packet::{Invocation, PacketStream};

use crate::collections::{initialize_native_collection, initialize_network_collection, initialize_wasm_collection};
use crate::dev::prelude::*;
use crate::json_writer::JsonWriter;
use crate::WAFL_V0_NAMESPACE;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) id: Uuid,
  pub(crate) manifest: ComponentConfiguration,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) timeout: Duration,
  pub(crate) rng_seed: Seed,
  pub(crate) namespace: Option<String>,
  pub(crate) event_log: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct NetworkService {
  #[allow(unused)]
  started_time: std::time::Instant,
  pub(crate) id: Uuid,
  interpreter: Arc<flow_graph_interpreter::Interpreter>,
}

type ServiceMap = HashMap<Uuid, Arc<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) async fn new(msg: Initialize) -> Result<Arc<Self>> {
    let graph = flow_graph_interpreter::graph::from_def(&msg.manifest)?;
    let mut collections = HandlerMap::default();
    let rng = Random::from_seed(msg.rng_seed);

    let stdlib = initialize_native_collection(WAFL_V0_NAMESPACE.to_owned(), rng.seed())?;

    collections.add(stdlib);

    for collection in msg.manifest.collections().values() {
      let collection_init = CollectionInitOptions {
        rng_seed: rng.seed(),
        network_id: msg.id,
        // mesh: msg.mesh.clone(),
        allow_latest: msg.allow_latest,
        allowed_insecure: msg.allowed_insecure.clone(),
        timeout: msg.timeout,
      };
      let p = initialize_collection(collection, collection_init).await?;
      collections.add(p);
    }

    let source = msg.manifest.source().clone();
    let mut interpreter = flow_graph_interpreter::Interpreter::new(
      Some(rng.seed()),
      graph,
      Some(msg.namespace.unwrap_or_else(|| msg.id.to_string())),
      Some(collections),
    )
    .map_err(|e| NetworkError::InterpreterInit(source.unwrap_or_else(|| "unknown".to_owned()), e))?;

    match msg.event_log {
      Some(path) => interpreter.start(None, Some(Box::new(JsonWriter::new(path)))).await,
      None => interpreter.start(None, None).await,
    }

    let network = Arc::new(NetworkService {
      started_time: std::time::Instant::now(),
      id: msg.id,
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(msg.id, network.clone());

    Ok(network)
  }

  pub(crate) fn new_from_manifest(
    uid: Uuid,
    location: &str,
    namespace: Option<String>,
    opts: CollectionInitOptions,
  ) -> BoxFuture<Result<Arc<NetworkService>>> {
    Box::pin(async move {
      let bytes = wick_loader_utils::get_bytes(location, opts.allow_latest, &opts.allowed_insecure).await?;
      let manifest = wick_config::ComponentConfiguration::load_from_bytes(Some(location.to_owned()), &bytes)?;

      let init = Initialize {
        id: uid,
        manifest,
        allowed_insecure: opts.allowed_insecure,
        allow_latest: opts.allow_latest,
        // mesh: opts.mesh,
        timeout: opts.timeout,
        rng_seed: opts.rng_seed,
        namespace,
        event_log: None,
      };
      NetworkService::new(init).await
    })
  }

  pub(crate) fn for_id(id: &Uuid) -> Option<Arc<Self>> {
    trace!(%id, "get network");
    let registry = HOST_REGISTRY.lock();
    registry.get(id).cloned()
  }

  pub(crate) async fn shutdown(&self) -> Result<()> {
    let _ = self.interpreter.shutdown().await;
    Ok(())
  }
}

impl InvocationHandler for NetworkService {
  fn get_signature(&self) -> std::result::Result<CollectionSignature, CollectionError> {
    let mut signature = self.interpreter.get_export_signature().clone();
    signature.name = Some(self.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    msg: Invocation,
    stream: PacketStream,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, CollectionError>>, CollectionError> {
    let tx_id = msg.tx_id;

    let fut = self.interpreter.invoke(msg, stream);

    Ok(
      async move {
        match fut.await {
          Ok(response) => Ok(InvocationResponse::Stream { tx_id, rx: response }),
          Err(e) => {
            error!("{}", e);
            Ok(InvocationResponse::error(
              tx_id,
              format!("Internal error invoking schematic: {}", e),
            ))
          }
        }
      }
      .boxed(),
    )
  }
}

#[derive(Debug)]
pub(crate) struct CollectionInitOptions {
  pub(crate) rng_seed: Seed,
  pub(crate) network_id: Uuid,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) timeout: Duration,
}

pub(crate) async fn initialize_collection(
  collection: &ComponentDefinition,
  opts: CollectionInitOptions,
) -> Result<NamespaceHandler> {
  let namespace = collection.namespace.clone();

  let result = match &collection.kind {
    ComponentKind::Wasm(v) => initialize_wasm_collection(v, namespace, opts).await,
    // CollectionKind::GrpcTar(v) => initialize_par_collection(v, namespace, opts).await,
    // CollectionKind::GrpcUrl(v) => initialize_grpc_collection(v, namespace).await,
    // CollectionKind::Mesh(v) => initialize_mesh_collection(v, namespace, opts).await,
    ComponentKind::Manifest(v) => initialize_network_collection(v, namespace, opts).await,
    _ => todo!(),
  };
  Ok(result?)
}

#[cfg(test)]
mod test {
  // You can find many of the network tests in the integration tests
}
