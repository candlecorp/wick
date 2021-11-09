pub(crate) mod error;
pub(crate) mod initialize;

use std::collections::HashMap;
use std::sync::Arc;

use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use vino_lattice::Lattice;
use vino_manifest::Loadable;

use crate::dev::prelude::validator::NetworkValidator;
use crate::dev::prelude::*;
use crate::network_service::initialize::{
  initialize_providers, initialize_schematics, start_schematic_services, start_self_network,
  update_providers, Initialize, ProviderInitOptions,
};

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]

pub(crate) struct NetworkService {
  #[allow(unused)]
  started_time: std::time::Instant,
  state: RwLock<Option<State>>,
  id: String,
}

#[derive(Debug)]
struct State {
  model: Arc<RwLock<NetworkModel>>,
  providers: HashMap<String, Arc<ProviderChannel>>,
  schematics: HashMap<String, Arc<SchematicService>>,
  lattice: Option<Arc<Lattice>>,
}

impl Default for NetworkService {
  fn default() -> Self {
    NetworkService {
      started_time: std::time::Instant::now(),
      id: "".to_owned(),
      state: RwLock::new(None),
    }
  }
}

type ServiceMap = HashMap<String, Arc<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) fn new<T: AsRef<str>>(id: T) -> Self {
    NetworkService {
      started_time: std::time::Instant::now(),
      id: id.as_ref().to_owned(),
      ..Default::default()
    }
  }

  pub(crate) async fn init(&self, msg: Initialize) -> Result<()> {
    trace!("NETWORK:INIT:{}", self.id);

    let global_providers = msg.network.providers.clone();
    let timeout = msg.timeout;

    let schematics = msg.network.schematics.clone();
    let address_map = start_schematic_services(&schematics);

    let model = Arc::new(RwLock::new(NetworkModel::try_from(msg.network.clone())?));

    let mut state = State {
      model: model.clone(),
      providers: HashMap::new(),
      schematics: address_map.clone(),
      lattice: msg.lattice,
    };

    let inner_model = model.clone();

    let provider_init = ProviderInitOptions {
      rng_seed: msg.rng_seed,
      network_id: self.id.clone(),
      lattice: state.lattice.clone(),
      allow_latest: msg.allow_latest,
      allowed_insecure: msg.allowed_insecure,
      timeout: msg.timeout,
    };

    let mut provider_map = HashMap::new();
    let mut model_map = HashMap::new();
    let self_channel = start_self_network(self.id.clone())?;
    provider_map.insert(SELF_NAMESPACE.to_owned(), Arc::new(self_channel));

    let providers = initialize_providers(global_providers, provider_init).await?;

    for (model, channel) in providers {
      model_map.insert(channel.namespace.clone(), model);
      provider_map.insert(channel.namespace.clone(), Arc::new(channel));
    }
    state.providers = provider_map.clone();

    self.state.write().replace(state);

    initialize_schematics(
      &inner_model,
      &address_map,
      timeout,
      &provider_map,
      &model_map,
    )?;
    self.finalize(model_map)?;
    Ok(())
  }

  pub(crate) fn finalize(&self, mut models: HashMap<String, ProviderModel>) -> Result<()> {
    let self_model = self.get_signature()?;

    let mut lock = self.state.write();
    let state = lock.as_mut().unwrap();
    models.insert(SELF_NAMESPACE.to_owned(), self_model.into());

    update_providers(&state.model, &models)?;

    state.model.write().finalize()?;
    NetworkValidator::validate(&state.model.write())?;
    Ok(())
  }

  pub(crate) fn for_id(uid: &str) -> Arc<Self> {
    trace!("NETWORK:GET:{}", uid);
    let mut registry = HOST_REGISTRY.lock();
    let network = registry.entry(uid.to_owned()).or_insert_with(|| {
      trace!("NETWORK:CREATE:{}", uid);
      Arc::new(NetworkService::new(uid))
    });
    network.clone()
  }

  // #[async_recursion::async_recursion]
  pub(crate) async fn init_from_manifest(
    &self,
    location: &str,
    opts: ProviderInitOptions,
  ) -> Result<()> {
    let bytes = vino_loader::get_bytes(location, opts.allow_latest, &opts.allowed_insecure).await?;
    let manifest = vino_manifest::HostManifest::load_from_bytes(&bytes)?;
    let def = NetworkDefinition::from(manifest.network());

    let init = Initialize {
      network: def,
      allowed_insecure: opts.allowed_insecure,
      allow_latest: opts.allow_latest,
      lattice: opts.lattice,
      timeout: opts.timeout,
      rng_seed: opts.rng_seed,
    };
    self.init(init).await
  }

  pub(crate) fn get_recipient(&self, entity: &Entity) -> Result<Arc<BoxedInvocationHandler>> {
    let err = Err(NetworkError::InvalidRecipient(entity.url()));
    let not_found = NetworkError::UnknownProvider(entity.url());
    let state_opt = self.state.read();
    let result = match state_opt.as_ref() {
      Some(state) => match &entity {
        Entity::Invalid => err,
        Entity::System(_) => err,
        Entity::Test(_) => err,
        Entity::Client(_) => err,
        Entity::Host(_) => err,
        Entity::Schematic(_) => state.providers.get(SELF_NAMESPACE).ok_or(not_found),
        Entity::Component(ns, _) => state.providers.get(ns).ok_or(not_found),
        Entity::Provider(name) => state.providers.get(name).ok_or(not_found),
        Entity::Reference(_) => err,
      },
      None => Err(NetworkError::Uninitialized),
    };

    result.map(|channel| channel.recipient.clone())
  }

  pub(crate) fn get_schematic_addr(&self, id: &str) -> Result<Arc<SchematicService>> {
    match self.state.read().as_ref() {
      Some(state) => state
        .schematics
        .get(id)
        .cloned()
        .ok_or_else(|| NetworkError::SchematicNotFound(id.to_owned())),
      None => Err(NetworkError::Uninitialized),
    }
  }
}

impl InvocationHandler for NetworkService {
  fn get_signature(&self) -> std::result::Result<ProviderSignature, ProviderError> {
    let state_opt = self.state.read();
    let state = match state_opt.as_ref() {
      Some(state) => state,
      None => return Err(ProviderError::Uninitialized(1001)),
    };

    let resolution_order = {
      let model = state.model.read();
      model
        .get_resolution_order()
        .map_err(|e| NetworkError::UnresolvableNetwork(e.to_string()))?
    };

    trace!(
      "NETWORK:RESOLUTION_ORDER:[{}]",
      join_comma(
        &resolution_order
          .iter()
          .map(|v| format!("[{}]", join_comma(v)))
          .collect::<Vec<_>>()
      )
    );

    let mut signatures = HashMap::new();
    for batch in resolution_order {
      for name in batch {
        trace!("NETWORK:SIGNATURE[{}]:REQUEST", name);
        let schematic_model = { state.model.read().get_schematic(&name).cloned() };

        match schematic_model {
          Some(schematic_model) => {
            let signature = {
              schematic_model
                .read()
                .get_signature()
                .cloned()
                .ok_or_else(|| {
                  NetworkError::UnresolvableNetwork(format!(
                    "Schematic '{}' does not have a signature",
                    name
                  ))
                })?
            };
            let mut scw = state.model.write();
            scw
              .update_self_component(name, signature.clone())
              .map_err(|e| NetworkError::InvalidState(e.to_string()))?;
            signatures.insert(signature.name.clone(), signature);
          }
          None => {
            return Err(
              NetworkError::InvalidState(format!(
                "Attempted to resolve schematic '{}' but '{}' is not running.",
                name, name
              ))
              .into(),
            );
          }
        }
      }
    }

    let provider_signature = ProviderSignature {
      name: Some(self.id.clone()),
      components: signatures.into(),
      types: StructMap::new(),
    };

    Ok(provider_signature)
  }

  fn invoke(
    &self,
    msg: InvocationMessage,
  ) -> std::result::Result<
    BoxFuture<std::result::Result<InvocationResponse, ProviderError>>,
    ProviderError,
  > {
    let tx_id = msg.get_tx_id().to_owned();

    let schematic_name = match msg.get_target() {
      Entity::Schematic(name) => name,
      Entity::Component(_, name) => name,
      _ => return Err(ProviderError::ComponentNotFound(msg.get_target_url())),
    };

    trace!("NETWORK[{}]:INVOKE:{}", self.id, schematic_name);
    let schematic = self
      .get_schematic_addr(schematic_name)
      .map_err(|e| ProviderError::ComponentNotFound(e.to_string()))?;

    Ok(
      async move {
        match schematic.invoke(&msg)?.await {
          Ok(response) => Ok(response),
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

#[cfg(test)]
mod test {
  // You can find many of the network tests in the integration tests
}
