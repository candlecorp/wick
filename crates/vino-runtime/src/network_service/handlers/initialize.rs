use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use parking_lot::RwLock;
use vino_lattice::lattice::Lattice;

use crate::dev::prelude::*;
use crate::providers::{
  initialize_grpc_provider,
  initialize_lattice_provider,
  initialize_network_provider,
  initialize_wasm_provider,
};
use crate::VINO_V0_NAMESPACE;
type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) network: NetworkDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) timeout: Duration,
  pub(crate) rng_seed: u64,
}

pub(crate) fn update_providers(
  model: &Arc<RwLock<NetworkModel>>,
  providers: &HashMap<String, ProviderChannel>,
) -> Result<()> {
  let mut map = HashMap::new();
  for (ns, channel) in providers {
    let ns = ns.clone();
    map.insert(ns, channel.model.clone());
  }
  let result = model.write().update_providers(map)?;
  Ok(result)
}

pub(crate) async fn start_self_network(nuid: String) -> Result<ProviderChannel> {
  trace!("NETWORK:PROVIDER:SELF:START");
  let self_channel = ProviderChannel {
    namespace: SELF_NAMESPACE.to_owned(),
    recipient: start_network_provider(nuid).await?,
    model: None,
  };
  trace!("NETWORK:PROVIDER:SELF:STARTED");
  Ok(self_channel)
}

pub(crate) async fn initialize_schematics(
  model: Arc<RwLock<NetworkModel>>,
  addresses: HashMap<String, Addr<SchematicService>>,
  timeout: Duration,
  providers: HashMap<String, ProviderChannel>,
) -> Result<()> {
  let schematics = model.read().get_schematics().clone();

  for model in schematics {
    let name = model.read().get_name();
    trace!("NETWORK:SCHEMATIC[{}]", name);
    let addr = addresses.get(&name).unwrap();
    addr
      .send(crate::schematic_service::handlers::initialize::Initialize {
        timeout,
        providers: providers.clone(),
        model,
      })
      .await
      .map_err(|_| InternalError::E5001)??;
    trace!("NETWORK:SCHEMATIC[{}]:INITIALIZED", name);
  }

  Ok(())
}

pub(crate) fn start_schematic_services(
  schematics: &[SchematicDefinition],
) -> HashMap<String, Addr<SchematicService>> {
  trace!("NETWORK:SCHEMATICS:STARTING");
  let result = map(schematics, |def| {
    // let arbiter = Arbiter::new();
    let arbiter = Arbiter::with_tokio_rt(|| tokio::runtime::Runtime::new().unwrap());
    let addr =
      SchematicService::start_in_arbiter(&arbiter.handle(), |_| SchematicService::default());
    (def.name.clone(), addr)
  });
  trace!("NETWORK:SCHEMATICS:STARTED");
  result
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn initialize_providers(
  providers: Vec<ProviderDefinition>,
  opts: ProviderInitOptions,
) -> BoxFuture<'static, Result<Vec<ProviderChannel>>> {
  Box::pin(async move {
    let channel = initialize_native_provider(VINO_V0_NAMESPACE, opts.rng_seed).await?;
    let mut channels = vec![channel];

    for provider in providers {
      let namespace = provider.namespace.clone();

      let result = match provider.kind {
        ProviderKind::Network => {
          initialize_network_provider(provider, namespace, opts.clone()).await
        }
        ProviderKind::Native => unreachable!(), // Should not be handled via this route
        ProviderKind::GrpcUrl => initialize_grpc_provider(provider, namespace).await,
        ProviderKind::Wapc => initialize_wasm_provider(provider, namespace, opts.clone()).await,
        ProviderKind::Lattice => {
          initialize_lattice_provider(provider, namespace, opts.clone()).await
        }
      };
      let channel = result?;
      channels.push(channel);
    }

    trace!("NETWORK:PROVIDERS:REGISTERED[{}]", channels.len());
    Ok(channels)
  })
}
