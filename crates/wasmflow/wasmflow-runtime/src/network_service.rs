pub(crate) mod error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wasmflow_interpreter::{HandlerMap, NamespaceHandler};
use wasmflow_lattice::Lattice;
use wasmflow_manifest::HostDefinition;

use crate::dev::prelude::*;
use crate::json_writer::JsonWriter;
use crate::providers::{
  initialize_grpc_provider,
  initialize_lattice_provider,
  initialize_native_provider,
  initialize_network_provider,
  initialize_par_provider,
  initialize_wasm_entrypoint,
  initialize_wasm_provider,
};
use crate::WAFL_V0_NAMESPACE;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) id: Uuid,
  pub(crate) manifest: HostDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) lattice: Option<Arc<Lattice>>,
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
  interpreter: Arc<wasmflow_interpreter::Interpreter>,
  entrypoint: Option<wasmflow_collection_wasm::provider::Provider>,
}

type ServiceMap = HashMap<Uuid, Arc<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) async fn new(msg: Initialize) -> Result<Arc<Self>> {
    let graph = wasmflow_interpreter::graph::from_def(msg.manifest.network())?;
    let mut providers = HandlerMap::default();
    let rng = Random::from_seed(msg.rng_seed);

    let stdlib = initialize_native_provider(WAFL_V0_NAMESPACE.to_owned(), rng.seed())?;

    providers.add(stdlib);

    for provider in &msg.manifest.network().providers {
      let provider_init = ProviderInitOptions {
        rng_seed: rng.seed(),
        network_id: msg.id,
        lattice: msg.lattice.clone(),
        allow_latest: msg.allow_latest,
        allowed_insecure: msg.allowed_insecure.clone(),
        timeout: msg.timeout,
      };
      let p = initialize_provider(provider, provider_init).await?;
      providers.add(p);
    }

    let source = msg.manifest.source.clone();
    let mut interpreter = wasmflow_interpreter::Interpreter::new(
      Some(rng.seed()),
      graph,
      Some(msg.namespace.unwrap_or_else(|| msg.id.to_string())),
      Some(providers),
    )
    .map_err(|e| NetworkError::InterpreterInit(source.unwrap_or_else(|| "unknown".to_owned()), e))?;

    match msg.event_log {
      Some(path) => interpreter.start(None, Some(Box::new(JsonWriter::new(path)))).await,
      None => interpreter.start(None, None).await,
    }

    let entrypoint = if let Some(entry) = &msg.manifest.network().entry {
      Some(initialize_wasm_entrypoint(entry, msg.id, msg.allow_latest, &msg.allowed_insecure).await?)
    } else {
      None
    };

    let network = Arc::new(NetworkService {
      started_time: std::time::Instant::now(),
      id: msg.id,
      entrypoint,
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(msg.id, network.clone());

    Ok(network)
  }

  pub(crate) async fn exec_main(&self, argv: Vec<String>) -> u32 {
    if let Some(entrypoint) = &self.entrypoint {
      entrypoint
        .exec_main(Entity::Collection(self.id.to_string()), argv)
        .await
    } else {
      error!("no entrypoint defined for network {}", self.id);
      99
    }
  }

  pub(crate) fn new_from_manifest(
    uid: Uuid,
    location: &str,
    namespace: Option<String>,
    opts: ProviderInitOptions,
  ) -> BoxFuture<Result<Arc<NetworkService>>> {
    Box::pin(async move {
      let bytes = wasmflow_loader::get_bytes(location, opts.allow_latest, &opts.allowed_insecure).await?;
      let manifest = wasmflow_manifest::HostDefinition::load_from_bytes(Some(location.to_owned()), &bytes)?;

      let init = Initialize {
        id: uid,
        manifest,
        allowed_insecure: opts.allowed_insecure,
        allow_latest: opts.allow_latest,
        lattice: opts.lattice,
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
  fn get_signature(&self) -> std::result::Result<ProviderSignature, ProviderError> {
    let mut signature = self.interpreter.get_export_signature().clone();
    signature.name = Some(self.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    msg: Invocation,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, ProviderError>>, ProviderError> {
    let tx_id = msg.tx_id;

    let fut = self.interpreter.invoke(msg);

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
pub(crate) struct ProviderInitOptions {
  pub(crate) rng_seed: Seed,
  pub(crate) network_id: Uuid,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) timeout: Duration,
}

pub(crate) async fn initialize_provider(
  provider: &ProviderDefinition,
  opts: ProviderInitOptions,
) -> Result<NamespaceHandler> {
  let namespace = provider.namespace.clone();

  let result = match provider.kind {
    ProviderKind::Network => initialize_network_provider(provider, namespace, opts).await,
    ProviderKind::Native => unreachable!(), // Should not be handled via this route
    ProviderKind::Par => initialize_par_provider(provider, namespace, opts).await,
    ProviderKind::GrpcUrl => initialize_grpc_provider(provider, namespace).await,
    ProviderKind::Wapc => initialize_wasm_provider(provider, namespace, opts).await,
    ProviderKind::Lattice => initialize_lattice_provider(provider, namespace, opts).await,
  };
  Ok(result?)
}

#[cfg(test)]
mod test {
  // You can find many of the network tests in the integration tests
}
