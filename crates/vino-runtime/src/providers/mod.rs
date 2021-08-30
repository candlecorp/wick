pub mod error;
pub(crate) mod grpc_provider_service;
pub(crate) mod native_provider_service;
pub(crate) mod network_provider;

use std::sync::Arc;

use vino_lattice::lattice::Lattice;

use self::native_provider_service::NativeProviderService;
use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, ProviderError>;

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

  let provider = Box::new(vino_native_api_0::Provider::default());
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
      address: provider.reference,
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
  namespace: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let component =
    vino_provider_wasm::helpers::load_wasm(&provider.reference, allow_latest, allowed_insecure)
      .await?;

  let provider = Box::new(vino_provider_wasm::provider::Provider::try_from_module(
    &component, 2,
  )?);

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

async fn initialize_lattice_provider(
  provider: ProviderDefinition,
  namespace: &str,
  lattice: Arc<Lattice>,
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let provider =
    Box::new(vino_provider_lattice::provider::Provider::new(provider.reference, lattice).await?);

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
  network_name: Option<String>,
  lattice: Option<Arc<Lattice>>,
  network_id: String,
) -> Result<Addr<NativeProviderService>> {
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  if let (Some(name), Some(lattice)) = (network_name, lattice) {
    let provider = Box::new(NetworkProvider::new(network_id.clone()));
    lattice.handle_namespace(name, move || provider).await?;
  }
  let provider = Box::new(NetworkProvider::new(network_id));

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: SELF_NAMESPACE.to_owned(),
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
  lattice: &Option<Arc<Lattice>>,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  trace!("Registering namespace {}", provider.namespace);
  let namespace = provider.namespace.clone();
  match provider.kind {
    ProviderKind::Native => unreachable!(), // Should not be handled via this route
    ProviderKind::GrpcUrl => initialize_grpc_provider(provider, seed, &namespace).await,
    ProviderKind::Wapc => {
      initialize_wasm_provider(provider, &namespace, allow_latest, allowed_insecure).await
    }
    ProviderKind::Lattice => match lattice {
      Some(lattice) => initialize_lattice_provider(provider, &namespace, lattice.clone()).await,
      None => Err(ProviderError::Lattice(
        "Attempted to initialize a lattice provider without an active lattice connection"
          .to_owned(),
      )),
    },
  }
}
