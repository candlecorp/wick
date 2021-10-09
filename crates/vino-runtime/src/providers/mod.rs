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
use crate::providers::grpc_provider_service::GrpcProviderService;

pub(crate) type BoxedInvocationHandler = Box<dyn InvocationHandler + Send + Sync>;

#[async_trait::async_trait]
pub(crate) trait InvocationHandler {
  async fn get_signature(&self) -> Result<ProviderSignature>;
  async fn invoke(&self, msg: InvocationMessage) -> Result<InvocationResponse>;
}

type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Clone)]
pub(crate) struct ProviderChannel {
  pub(crate) namespace: String,
  pub(crate) recipient: Arc<BoxedInvocationHandler>,
  pub(crate) model: Option<ProviderModel>,
}

impl std::fmt::Debug for ProviderChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ProviderChannel")
      .field("namespace", &self.namespace)
      .field("model", &self.model)
      .finish()
  }
}

pub(crate) async fn initialize_native_provider(
  namespace: &str,
  seed: u64,
) -> Result<ProviderChannel> {
  trace!("PROV:NATIVE:NS[{}]:REGISTERING", namespace);
  let provider = Box::new(vino_native_api_0::Provider::new(seed));
  let service = NativeProviderService::new(namespace.to_owned(), provider);

  let signature = service.get_signature().await?;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: Arc::new(Box::new(service)),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_grpc_provider(
  provider: ProviderDefinition,
  seed: &str,
  namespace: &str,
) -> Result<ProviderChannel> {
  trace!("PROV:GRPC:NS[{}]:REGISTERING", provider.namespace);

  let mut service = GrpcProviderService::new(namespace.to_owned(), seed.to_owned());
  service.init(provider.reference.clone()).await?;

  let signature = service.get_signature().await?;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: Arc::new(Box::new(service)),
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

  let component =
    vino_provider_wasm::helpers::load_wasm(&provider.reference, allow_latest, allowed_insecure)
      .await?;

  // TODO need to propagate wasi params from manifest
  let provider = Box::new(vino_provider_wasm::provider::Provider::try_load(
    &component,
    Some(provider.data.clone()),
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

  let service = NativeProviderService::new(namespace.to_owned(), provider);

  let signature = service.get_signature().await?;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: Arc::new(Box::new(service)),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_network_provider<'a>(
  provider: ProviderDefinition,
  namespace: &'a str,
  opts: NetworkOptions<'a>,
) -> Result<ProviderChannel> {
  trace!("PROV:NETWORK:NS[{}]:REGISTERING", provider.namespace);

  let network_id: String = NetworkService::start_from_manifest(
    &provider.reference,
    opts.rng_seed,
    opts.allow_latest,
    opts.insecure.to_owned(),
    opts.lattice.clone(),
    opts.timeout,
  )
  .await
  .map_err(|e| ProviderError::SubNetwork(e.to_string()))?;

  let provider = Box::new(network_provider::Provider::new(network_id));

  let service = NativeProviderService::new(namespace.to_owned(), provider);

  let signature = service.get_signature().await?;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: Arc::new(Box::new(service)),
    model: Some(signature.into()),
  })
}

pub(crate) async fn initialize_lattice_provider(
  provider: ProviderDefinition,
  namespace: &str,
  lattice: Arc<Lattice>,
) -> Result<ProviderChannel> {
  trace!("PROV:LATTICE:NS[{}]:REGISTERING", provider.namespace);

  let provider =
    Box::new(vino_provider_lattice::provider::Provider::new(provider.reference, lattice).await?);

  let service = NativeProviderService::new(namespace.to_owned(), provider);

  let signature = service.get_signature().await?;

  Ok(ProviderChannel {
    namespace: namespace.to_owned(),
    recipient: Arc::new(Box::new(service)),
    model: Some(signature.into()),
  })
}

pub(crate) async fn start_network_provider(
  network_id: String,
) -> Result<Arc<BoxedInvocationHandler>> {
  trace!("PROV:NETWORK[{}]", network_id);

  let provider = Box::new(NetworkProvider::new(network_id));

  let service = NativeProviderService::new(SELF_NAMESPACE.to_owned(), provider);

  Ok::<Arc<BoxedInvocationHandler>, ProviderError>(Arc::new(Box::new(service)))
}

pub(crate) async fn create_network_provider_model(
  service: Arc<BoxedInvocationHandler>,
) -> Result<ProviderSignature> {
  let signature = service.get_signature().await?;

  Ok(signature.into())
}

pub(crate) struct NetworkOptions<'a> {
  pub(crate) rng_seed: u64,
  pub(crate) lattice: &'a Option<Arc<Lattice>>,
  pub(crate) allow_latest: bool,
  pub(crate) insecure: &'a [String],
  pub(crate) timeout: Duration,
}
