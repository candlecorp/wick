use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::dev::prelude::*;
use crate::error::ComponentError;
use crate::providers::network_provider_service::{
  self,
  NetworkProviderService,
};
use crate::providers::wapc_module::load_component;
use crate::providers::{
  grpc_provider_service,
  native_provider_service,
  wapc_provider_service,
};
type Result<T> = std::result::Result<T, ComponentError>;

#[derive(Debug, Clone)]
pub(crate) struct ProviderModel {
  pub(crate) namespace: String,
  pub(crate) components: HashMap<String, ComponentModel>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderChannel {
  pub(crate) namespace: String,
  pub(crate) recipient: Recipient<Invocation>,
}

pub(crate) async fn initialize_native_provider(
  namespace: &str,
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let provider = Arc::new(Mutex::new(vino_native_components_v0::Provider::default()));
  let addr = native_provider_service::NativeProviderService::start_in_arbiter(&handle, |_| {
    native_provider_service::NativeProviderService::default()
  });
  let components = addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  Ok((
    ProviderChannel {
      namespace: namespace.to_owned(),
      recipient: addr.recipient(),
    },
    ProviderModel {
      namespace: namespace.to_owned(),
      components,
    },
  ))
}

async fn initialize_grpc_provider(
  provider: ProviderDefinition,
  seed: &str,
  namespace: &str,
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let addr = grpc_provider_service::GrpcProviderService::start_in_arbiter(&handle, |_| {
    grpc_provider_service::GrpcProviderService::default()
  });

  let components = addr
    .send(grpc_provider_service::Initialize {
      namespace: namespace.to_owned(),
      address: provider.reference.clone(),
      signing_seed: seed.to_owned(),
    })
    .await??;

  Ok((
    ProviderChannel {
      namespace: namespace.to_owned(),
      recipient: addr.recipient(),
    },
    ProviderModel {
      namespace: namespace.to_owned(),
      components,
    },
  ))
}

async fn initialize_wapc_provider(
  provider: ProviderDefinition,
  seed: &str,
  namespace: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let addr = wapc_provider_service::WapcProviderService::start_in_arbiter(&handle, |_| {
    wapc_provider_service::WapcProviderService::default()
  });

  let component =
    load_component(provider.reference.clone(), allow_latest, allowed_insecure).await?;

  let components = addr
    .send(wapc_provider_service::Initialize {
      namespace: namespace.to_owned(),
      signing_seed: seed.to_owned(),
      claims: component.claims(),
      bytes: component.bytes,
    })
    .await??;

  Ok((
    ProviderChannel {
      namespace: namespace.to_owned(),
      recipient: addr.recipient(),
    },
    ProviderModel {
      namespace: namespace.to_owned(),
      components,
    },
  ))
}

pub(crate) async fn start_network_provider(
  namespace: &str,
  network_id: String,
) -> Result<Addr<NetworkProviderService>> {
  trace!("Starting network provider");
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let provider = Arc::new(Mutex::new(NetworkProvider::new(network_id)));

  let addr =
    NetworkProviderService::start_in_arbiter(&handle, |_| NetworkProviderService::default());
  addr
    .send(network_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;
  Ok(addr)
}

pub(crate) async fn create_network_provider_model(
  namespace: &str,
  addr: Addr<NetworkProviderService>,
) -> Result<ProviderModel> {
  let components = addr
    .send(network_provider_service::InitializeComponents {})
    .await??;

  Ok(ProviderModel {
    namespace: namespace.to_owned(),
    components,
  })
}

pub(crate) async fn initialize_provider(
  provider: ProviderDefinition,
  seed: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  let namespace = provider.namespace.clone();
  trace!("Registering provider under the namespace {}", namespace);
  match provider.kind {
    ProviderKind::Native => unreachable!(), ///// Should not be handled via this route
    ProviderKind::Schematic => unreachable!(), // same
    ProviderKind::GrpcUrl => initialize_grpc_provider(provider, seed, &namespace).await,
    ProviderKind::Wapc => {
      initialize_wapc_provider(provider, seed, &namespace, allow_latest, allowed_insecure).await
    }
  }
}
