use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use parking_lot::RwLock;
use vino_lattice::Lattice;

use crate::dev::prelude::*;
use crate::providers::{
  initialize_grpc_provider, initialize_lattice_provider, initialize_network_provider,
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
  network_model: &Arc<RwLock<NetworkModel>>,
  models: &HashMap<String, ProviderModel>,
) -> Result<()> {
  let mut map = HashMap::new();
  for (ns, model) in models {
    let ns = ns.clone();
    map.insert(ns, model.clone());
  }
  let result = network_model.write().update_providers(map)?;
  Ok(result)
}

pub(crate) fn start_self_network(nuid: String) -> Result<ProviderChannel> {
  trace!("NETWORK:PROVIDER:SELF:START");
  let self_channel = ProviderChannel {
    namespace: SELF_NAMESPACE.to_owned(),
    recipient: start_network_provider(nuid)?,
  };
  trace!("NETWORK:PROVIDER:SELF:STARTED");
  Ok(self_channel)
}

pub(crate) fn initialize_schematics(
  model: &Arc<RwLock<NetworkModel>>,
  services: &HashMap<String, Arc<SchematicService>>,
  timeout: Duration,
  provider_channels: &HashMap<String, Arc<ProviderChannel>>,
  provider_models: &HashMap<String, ProviderModel>,
) -> Result<()> {
  let schematics = model.read().get_schematics().clone();

  for model in schematics {
    let name = model.read().get_name();
    trace!("NETWORK:SCHEMATIC[{}]", name);
    let schematic = services.get(&name).unwrap();
    schematic.init(&model, provider_channels, provider_models.clone(), timeout)?;
    trace!("NETWORK:SCHEMATIC[{}]:INITIALIZED", name);
  }

  Ok(())
}

pub(crate) fn start_schematic_services(
  schematics: &[SchematicDefinition],
) -> HashMap<String, Arc<SchematicService>> {
  trace!("NETWORK:SCHEMATICS:STARTING");
  let result = map(schematics, |def| {
    let name = def.name.clone();
    let service = SchematicService::new(def);
    (name, Arc::new(service))
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
) -> BoxFuture<'static, Result<Vec<(ProviderModel, ProviderChannel)>>> {
  Box::pin(async move {
    let channel = initialize_native_provider(VINO_V0_NAMESPACE.to_owned(), opts.rng_seed)?;
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
