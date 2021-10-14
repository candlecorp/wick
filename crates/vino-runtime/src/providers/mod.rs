pub mod error;
pub(crate) mod grpc_provider_service;
pub(crate) mod native_provider_service;
pub(crate) mod network_provider;

use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use vino_provider_wasm::error::LinkError;

use self::native_provider_service::NativeProviderService;
use crate::dev::prelude::*;
use crate::dispatch::network_invoke_sync;
use crate::network_service::initialize::ProviderInitOptions;
use crate::providers::grpc_provider_service::GrpcProviderService;

pub(crate) type BoxedInvocationHandler = Box<dyn InvocationHandler + Send + Sync>;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ProviderSignature>;
  fn invoke(&self, msg: InvocationMessage) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Clone)]
pub(crate) struct ProviderChannel {
  pub(crate) namespace: String,
  pub(crate) recipient: Arc<BoxedInvocationHandler>,
  // pub(crate) model: Option<ProviderModel>,
}

impl std::fmt::Debug for ProviderChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ProviderChannel")
      .field("namespace", &self.namespace)
      .finish()
  }
}

type ProviderInitResult = Result<(ProviderModel, ProviderChannel)>;

pub(crate) async fn initialize_native_provider(namespace: String, seed: u64) -> ProviderInitResult {
  trace!("PROV:NATIVE:NS[{}]:REGISTERING", namespace);
  let provider = Arc::new(vino_native_api_0::Provider::new(seed));
  let service = NativeProviderService::new(namespace.clone(), provider);

  let signature = service.get_signature()?;

  Ok((
    signature.into(),
    ProviderChannel {
      namespace,
      recipient: Arc::new(Box::new(service)),
    },
  ))
}

pub(crate) async fn initialize_grpc_provider(
  provider: ProviderDefinition,
  namespace: String,
) -> ProviderInitResult {
  trace!("PROV:GRPC:NS[{}]:REGISTERING", provider.namespace);

  let mut service = GrpcProviderService::new(namespace.clone());
  service.init(provider.reference.clone()).await?;

  let signature = service.get_signature()?;

  Ok((
    signature.into(),
    ProviderChannel {
      namespace: namespace.clone(),
      recipient: Arc::new(Box::new(service)),
    },
  ))
}

pub(crate) async fn initialize_wasm_provider(
  provider: ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!("PROV:WASM:NS[{}]:REGISTERING", provider.namespace);

  let component = vino_provider_wasm::helpers::load_wasm(
    &provider.reference,
    opts.allow_latest,
    &opts.allowed_insecure,
  )
  .await?;

  // TODO need to propagate wasi params from manifest
  let provider = Arc::new(vino_provider_wasm::provider::Provider::try_load(
    &component,
    Some(provider.data.clone()),
    None,
    Some(Box::new(move |origin_url, target_url, payload| {
      debug!(
        "PROV:WASM:LINK_CALL[{} => {}]:NETWORK[{}]",
        origin_url, target_url, opts.network_id
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
      let result = network_invoke_sync(opts.network_id.clone(), origin, target, payload)
        .map_err(|e| LinkError::CallFailure(e.to_string()))?;
      Ok(result)
    })),
  )?);

  let service = NativeProviderService::new(namespace.clone(), provider);

  let signature = service.get_signature()?;

  Ok((
    signature.into(),
    ProviderChannel {
      namespace: namespace.clone(),
      recipient: Arc::new(Box::new(service)),
    },
  ))
}

pub(crate) async fn initialize_network_provider(
  provider: ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!("PROV:NETWORK:NS[{}]:REGISTERING", provider.namespace);
  let kp = KeyPair::new_server().public_key();

  let network = NetworkService::for_id(&kp);
  network
    .init_from_manifest(&provider.reference, opts)
    .await
    .map_err(|e| ProviderError::SubNetwork(e.to_string()))?;

  let provider = Arc::new(network_provider::Provider::new(kp));

  let service = NativeProviderService::new(namespace.clone(), provider);

  let signature = service.get_signature()?;

  Ok((
    signature.into(),
    ProviderChannel {
      namespace: namespace.clone(),
      recipient: Arc::new(Box::new(service)),
    },
  ))
}

pub(crate) async fn initialize_lattice_provider(
  provider: ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  let lattice = match opts.lattice {
    Some(lattice) => lattice,
    None => {
      return Err(ProviderError::Lattice(
        "Lattice provider defined but no lattice available".to_owned(),
      ))
    }
  };
  trace!("PROV:LATTICE:NS[{}]:REGISTERING", provider.namespace);

  let provider =
    Arc::new(vino_provider_lattice::provider::Provider::new(provider.reference, lattice).await?);

  let service = NativeProviderService::new(namespace.clone(), provider);

  let signature = service.get_signature()?;

  Ok((
    signature.into(),
    ProviderChannel {
      namespace: namespace.clone(),
      recipient: Arc::new(Box::new(service)),
    },
  ))
}

pub(crate) async fn start_network_provider(
  network_id: String,
) -> Result<Arc<BoxedInvocationHandler>> {
  trace!("PROV:NETWORK[{}]", network_id);

  let provider = Arc::new(NetworkProvider::new(network_id));

  let service = NativeProviderService::new(SELF_NAMESPACE.to_owned(), provider);

  Ok::<Arc<BoxedInvocationHandler>, ProviderError>(Arc::new(Box::new(service)))
}
