use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::dev::prelude::*;
use crate::error::ProviderError;
use crate::providers::native_provider_service::NativeProviderService;
use crate::providers::{
  grpc_provider_service,
  native_provider_service,
};
type Result<T> = std::result::Result<T, ProviderError>;

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

  let provider = Arc::new(Mutex::new(vino_native_api_0::Provider::default()));
  let addr = native_provider_service::NativeProviderService::start_in_arbiter(&handle, |_| {
    native_provider_service::NativeProviderService::default()
  });
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  let components = addr
    .send(native_provider_service::InitializeComponents {})
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

async fn initialize_wasm_provider(
  provider: ProviderDefinition,
  _seed: &str,
  namespace: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let component =
    vino_provider_wasm::helpers::load_wasm(&provider.reference, allow_latest, allowed_insecure)
      .await?;

  let provider = Arc::new(Mutex::new(
    vino_provider_wasm::provider::Provider::try_from_module(component, 2)?,
  ));

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  let components = addr
    .send(native_provider_service::InitializeComponents {})
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
) -> Result<Addr<NativeProviderService>> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let provider = Arc::new(Mutex::new(NetworkProvider::new(network_id)));

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;
  Ok(addr)
}

pub(crate) async fn create_network_provider_model(
  namespace: &str,
  addr: Addr<NativeProviderService>,
) -> Result<ProviderModel> {
  let components = addr
    .send(native_provider_service::InitializeComponents {})
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
  trace!("PRV:Registering namespace {}", namespace);
  match provider.kind {
    ProviderKind::Native => unreachable!(), // Should not be handled via this route
    ProviderKind::GrpcUrl => initialize_grpc_provider(provider, seed, &namespace).await,
    ProviderKind::Wapc => {
      initialize_wasm_provider(provider, seed, &namespace, allow_latest, allowed_insecure).await
    }
  }
}
