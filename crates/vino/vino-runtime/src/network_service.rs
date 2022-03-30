pub(crate) mod error;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_interpreter::{ProviderNamespace, Providers};
use vino_lattice::Lattice;
use vino_manifest::HostDefinition;

use crate::dev::prelude::*;
use crate::providers::{
  initialize_grpc_provider,
  initialize_lattice_provider,
  initialize_native_provider,
  initialize_network_provider,
  initialize_par_provider,
  initialize_runtime_core,
  initialize_wasm_provider,
};
use crate::VINO_V0_NAMESPACE;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]

pub(crate) struct NetworkService {
  #[allow(unused)]
  started_time: std::time::Instant,
  id: String,
  interpreter: Arc<vino_interpreter::Interpreter>,
}

type ServiceMap = HashMap<String, Arc<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) async fn new(msg: Initialize) -> Result<Arc<Self>> {
    let graph = vino_interpreter::graph::from_def(msg.network.network())?;
    let mut providers = Providers::default();

    let provider_init = ProviderInitOptions {
      rng_seed: msg.rng_seed,
      network_id: msg.id.clone(),
      lattice: msg.lattice.clone(),
      allow_latest: msg.allow_latest,
      allowed_insecure: msg.allowed_insecure,
      timeout: msg.timeout,
    };
    let stdlib = initialize_native_provider(VINO_V0_NAMESPACE.to_owned(), msg.rng_seed)?;
    providers.add(stdlib);
    providers.add(initialize_runtime_core()?);
    for provider in &msg.network.network().providers {
      let p = initialize_provider(provider, provider_init.clone()).await?;
      providers.add(p);
    }
    let mut interpreter = vino_interpreter::Interpreter::new(graph, Some(providers))?;
    interpreter.start().await;

    let network = Arc::new(NetworkService {
      started_time: std::time::Instant::now(),
      id: msg.id.clone(),
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(msg.id, network.clone());

    Ok(network)
  }

  pub(crate) fn new_from_manifest<T: AsRef<str> + Send + 'static>(
    uid: T,
    location: &str,
    opts: ProviderInitOptions,
  ) -> BoxFuture<Result<Arc<NetworkService>>> {
    Box::pin(async move {
      let uid = uid.as_ref().to_owned();
      let bytes = vino_loader::get_bytes(location, opts.allow_latest, &opts.allowed_insecure).await?;
      let manifest = vino_manifest::HostDefinition::load_from_bytes(&bytes)?;

      let init = Initialize {
        id: uid.clone(),
        network: manifest,
        allowed_insecure: opts.allowed_insecure,
        allow_latest: opts.allow_latest,
        lattice: opts.lattice,
        timeout: opts.timeout,
        rng_seed: opts.rng_seed,
      };
      NetworkService::new(init).await
    })
  }

  pub(crate) fn for_id(uid: &str) -> Option<Arc<Self>> {
    trace!(uid, "get network");
    let registry = HOST_REGISTRY.lock();
    registry.get(uid).cloned()
  }
}

impl InvocationHandler for NetworkService {
  fn get_signature(&self) -> std::result::Result<ProviderSignature, ProviderError> {
    let mut signature = self.interpreter.get_export_signature().clone();
    signature.name = Some(self.id.clone());

    Ok(signature)
  }

  fn invoke(
    &self,
    msg: Invocation,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, ProviderError>>, ProviderError> {
    let tx_id = msg.tx_id.clone();

    let fut = self.interpreter.invoke(msg);

    Ok(
      async move {
        match fut.await {
          Ok(response) => Ok(InvocationResponse::Stream { tx_id, rx: response }),
          Err(e) => Ok(InvocationResponse::error(
            tx_id,
            format!("Internal error invoking schematic: {}", e),
          )),
        }
      }
      .boxed(),
    )
  }
}

#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) id: String,
  pub(crate) network: HostDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) timeout: Duration,
  pub(crate) rng_seed: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderInitOptions {
  pub(crate) rng_seed: u64,
  pub(crate) network_id: String,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) timeout: Duration,
}

pub(crate) async fn initialize_provider(
  provider: &ProviderDefinition,
  opts: ProviderInitOptions,
) -> Result<ProviderNamespace> {
  let namespace = provider.namespace.clone();

  let result = match provider.kind {
    ProviderKind::Network => initialize_network_provider(provider, namespace, opts.clone()).await,
    ProviderKind::Native => unreachable!(), // Should not be handled via this route
    ProviderKind::Par => initialize_par_provider(provider, namespace, opts.clone()).await,
    ProviderKind::GrpcUrl => initialize_grpc_provider(provider, namespace).await,
    ProviderKind::Wapc => initialize_wasm_provider(provider, namespace, opts.clone()).await,
    ProviderKind::Lattice => initialize_lattice_provider(provider, namespace, opts.clone()).await,
  };
  Ok(result?)
}

#[cfg(test)]
mod test {
  // You can find many of the network tests in the integration tests
}
