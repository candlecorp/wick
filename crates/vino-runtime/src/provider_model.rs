use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::components::network_provider_service::{
  self,
  NetworkProviderService,
};
use crate::components::vino_component::{
  load_component,
  VinoComponent,
};
use crate::components::{
  grpc_url_provider,
  native_provider,
  wapc_provider,
};
use crate::dev::prelude::*;
use crate::error::ComponentError;
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

  let provider = Arc::new(Mutex::new(vino_native_provider::Provider::default()));
  let addr = native_provider::NativeProvider::start_in_arbiter(&handle, |_| {
    native_provider::NativeProvider::default()
  });
  let components = addr
    .send(native_provider::Initialize {
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

  let addr = grpc_url_provider::GrpcUrlProvider::start_in_arbiter(&handle, |_| {
    grpc_url_provider::GrpcUrlProvider::default()
  });

  let components = addr
    .send(grpc_url_provider::Initialize {
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

  let addr = wapc_provider::WapcProvider::start_in_arbiter(&handle, |_| {
    wapc_provider::WapcProvider::default()
  });

  let component =
    load_component(provider.reference.clone(), allow_latest, allowed_insecure).await?;

  let components = addr
    .send(wapc_provider::Initialize {
      namespace: namespace.to_owned(),
      signing_seed: seed.to_owned(),
      name: component.name(),
      outputs: component.get_outputs(),
      inputs: component.get_inputs(),
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

pub(crate) async fn create_network_provider_channel(
  namespace: &str,
  network_id: String,
) -> Result<ProviderChannel> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let provider = Arc::new(Mutex::new(crate::NetworkProvider::new(network_id)));

  let addr =
    NetworkProviderService::start_in_arbiter(&handle, |_| NetworkProviderService::default());
  addr
    .send(network_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;
  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
  })
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
