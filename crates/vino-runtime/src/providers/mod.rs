pub mod error;
pub(crate) mod grpc_provider_service;
pub(crate) mod native_provider_service;
pub(crate) mod network_provider;

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use vino_lattice::lattice::Lattice;
use vino_provider_wasm::error::LinkError;

use self::native_provider_service::NativeProviderService;
use crate::dev::prelude::*;
use crate::dispatch::network_invoke_sync;

type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug, Clone)]
pub(crate) struct ProviderChannel {
  pub(crate) namespace: String,
  pub(crate) recipient: Recipient<InvocationMessage>,
  pub(crate) model: Option<ProviderModel>,
}

pub(crate) async fn initialize_native_provider(
  namespace: &str,
  seed: u64,
) -> Result<ProviderChannel> {
  trace!("PROV:NATIVE:NS[{}]:REGISTERING", namespace);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let provider = Box::new(vino_native_api_0::Provider::new(seed));
  let addr = native_provider_service::NativeProviderService::start_in_arbiter(&handle, |_| {
    native_provider_service::NativeProviderService::default()
  });
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  let signature = addr
    .send(native_provider_service::InitializeComponents {})
    .await??;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_grpc_provider(
  provider: ProviderDefinition,
  seed: &str,
  namespace: &str,
) -> Result<ProviderChannel> {
  trace!("PROV:GRPC:NS[{}]:REGISTERING", provider.namespace);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let addr = grpc_provider_service::GrpcProviderService::start_in_arbiter(&handle, |_| {
    grpc_provider_service::GrpcProviderService::default()
  });

  let signature = addr
    .send(grpc_provider_service::Initialize {
      namespace: namespace.to_owned(),
      address: provider.reference,
      signing_seed: seed.to_owned(),
    })
    .await??;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_wasm_provider(
  provider: ProviderDefinition,
  namespace: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
  network_id: String,
) -> Result<ProviderChannel> {
  trace!("PROV:WASM:NS[{}]:REGISTERING", provider.namespace);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();
  let component =
    vino_provider_wasm::helpers::load_wasm(&provider.reference, allow_latest, allowed_insecure)
      .await?;

  // TODO need to propagate wasi params from manifest
  let provider = Box::new(vino_provider_wasm::provider::Provider::try_load(
    &component,
    2,
    None,
    None,
    Some(Box::new(move |origin_url, target_url, payload| {
      debug!(
        "PROV:WASM:LINK_CALL[{} => {}]:NETWORK[{}]",
        origin_url, target_url, network_id
      );
      let target = Entity::from_str(target_url)?;
      let origin = Entity::from_str(origin_url)?;
      if let Entity::Component(origin_ns, _) = &origin {
        if let Entity::Component(target_ns, _) = &target {
          if target_ns == origin_ns {
            return Err(LinkError::Circular(target_ns.clone()));
          }
        }
      }
      let result = network_invoke_sync(network_id.clone(), origin, target, payload)
        .map_err(|e| LinkError::CallFailure(e.to_string()))?;
      Ok(result)
    })),
  )?);

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  let signature = addr
    .send(native_provider_service::InitializeComponents {})
    .await??;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_network_provider<'a>(
  provider: ProviderDefinition,
  namespace: &'a str,
  opts: NetworkOptions<'a>,
) -> Result<ProviderChannel> {
  trace!("PROV:NETWORK:NS[{}]:REGISTERING", provider.namespace);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let network_id: String = NetworkService::start_from_manifest(
    &provider.reference,
    opts.rng_seed,
    opts.seed,
    opts.allow_latest,
    opts.insecure.to_owned(),
    opts.lattice.clone(),
    opts.timeout,
  )
  .await
  .map_err(|e| ProviderError::SubNetwork(e.to_string()))?;

  let provider = Box::new(network_provider::Provider::new(network_id));

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider: provider.clone(),
      namespace: namespace.to_owned(),
    })
    .await??;

  let signature = addr
    .send(native_provider_service::InitializeComponents {})
    .await??;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_lattice_provider(
  provider: ProviderDefinition,
  namespace: &str,
  lattice: Arc<Lattice>,
) -> Result<ProviderChannel> {
  trace!("PROV:LATTICE:NS[{}]:REGISTERING", provider.namespace);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

  let provider =
    Box::new(vino_provider_lattice::provider::Provider::new(provider.reference, lattice).await?);

  let addr = NativeProviderService::start_in_arbiter(&handle, |_| NativeProviderService::default());
  addr
    .send(native_provider_service::Initialize {
      provider,
      namespace: namespace.to_owned(),
    })
    .await??;

  let signature = addr
    .send(native_provider_service::InitializeComponents {})
    .await??;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: addr.recipient(),
    model: Some(signature.into()),
  })
}

pub(crate) async fn start_network_provider(
  network_id: String,
) -> Result<Addr<NativeProviderService>> {
  trace!("PROV:NETWORK[{}]", network_id);
  let arbiter = Arbiter::new();
  let handle = arbiter.handle();

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
  addr: Addr<NativeProviderService>,
) -> Result<ProviderSignature> {
  let signature = addr
    .send(native_provider_service::InitializeComponents {})
    .await??;

  Ok(signature.into())
}

pub(crate) struct NetworkOptions<'a> {
  pub(crate) rng_seed: u64,
  pub(crate) seed: &'a str,
  pub(crate) lattice: &'a Option<Arc<Lattice>>,
  pub(crate) allow_latest: bool,
  pub(crate) insecure: &'a [String],
  pub(crate) timeout: Duration,
}
