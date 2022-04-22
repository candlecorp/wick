pub(crate) mod error;
pub(crate) mod native_provider_service;
pub(crate) mod network_provider;

use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use uuid::Uuid;
use vino_interpreter::NamespaceHandler;
use vino_manifest::network_definition::EntrypointDefinition;
use vino_provider_wasm::error::LinkError;
use vino_provider_wasm::provider::HostLinkCallback;
use vino_random::{Random, Seed};

use self::native_provider_service::NativeProviderService;
use crate::dev::prelude::*;
use crate::dispatch::network_invoke_sync;
use crate::network_service::ProviderInitOptions;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ProviderSignature>;
  fn invoke(&self, msg: Invocation) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ProviderError>;

type ProviderInitResult = Result<NamespaceHandler>;

#[instrument(skip(provider, opts))]
pub(crate) async fn initialize_par_provider(
  provider: &ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!(namespace = %provider.namespace, ?opts, "registering");

  let bytes = vino_loader::get_bytes(&provider.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  let service = vino_provider_par::provider::Provider::from_tarbytes(
    provider.reference.clone(),
    &*bytes,
    Some(provider.data.clone()),
  )
  .await?;

  let service = NativeProviderService::new(Arc::new(service));

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(provider))]
pub(crate) async fn initialize_grpc_provider(provider: &ProviderDefinition, namespace: String) -> ProviderInitResult {
  trace!(namespace = %provider.namespace, "registering");

  let service = vino_provider_grpc::provider::Provider::new(provider.reference.clone()).await?;

  let service = NativeProviderService::new(Arc::new(service));

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(provider, opts))]
pub(crate) async fn initialize_wasm_provider(
  provider: &ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!(namespace = %provider.namespace, ?opts, "registering");

  let component =
    vino_provider_wasm::helpers::load_wasm(&provider.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  // TODO take max threads from configuration
  let provider = Arc::new(vino_provider_wasm::provider::Provider::try_load(
    &component,
    5,
    Some(provider.data.clone()),
    None,
    Some(make_link_callback(opts.network_id)),
  )?);

  let service = NativeProviderService::new(provider);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip_all)]
pub(crate) async fn initialize_wasm_entrypoint(
  entrypoint: &EntrypointDefinition,
  network_id: Uuid,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<vino_provider_wasm::provider::Provider> {
  trace!(%network_id, "registering entrypoint");

  let component = vino_provider_wasm::helpers::load_wasm(&entrypoint.reference, allow_latest, allowed_insecure).await?;

  // TODO take max threads from configuration
  let provider = vino_provider_wasm::provider::Provider::try_load(
    &component,
    1,
    Some(entrypoint.data.clone()),
    None,
    Some(make_link_callback(network_id)),
  )?;

  Ok(provider)
}

fn make_link_callback(network_id: Uuid) -> Box<HostLinkCallback> {
  Box::new(move |origin_url, target_url, payload| {
    debug!(
      origin = origin_url,
      target = target_url,
      network_id = %network_id,
      "link_call"
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
    let invocation = Invocation::new(origin, target, payload, None);
    let result = network_invoke_sync(network_id, invocation).map_err(|e| LinkError::CallFailure(e.to_string()))?;
    Ok(result)
  })
}

#[instrument(skip(provider, opts))]
pub(crate) async fn initialize_network_provider(
  provider: &ProviderDefinition,
  namespace: String,
  mut opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!(namespace = %provider.namespace, ?opts, "registering");

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();

  let _network = NetworkService::new_from_manifest(uuid, &provider.reference, Some(namespace.clone()), opts)
    .await
    .map_err(|e| ProviderError::SubNetwork(provider.reference.clone(), e.to_string()))?;

  let provider = Arc::new(network_provider::Provider::new(uuid));

  let service = NativeProviderService::new(provider);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(provider, opts))]
pub(crate) async fn initialize_lattice_provider(
  provider: &ProviderDefinition,
  namespace: String,
  opts: ProviderInitOptions,
) -> ProviderInitResult {
  trace!(namespace = %provider.namespace, ?opts, "registering");
  let lattice = match opts.lattice {
    Some(lattice) => lattice,
    None => {
      return Err(ProviderError::Lattice(
        "Lattice provider defined but no lattice available".to_owned(),
      ))
    }
  };

  let provider = Arc::new(vino_provider_lattice::provider::Provider::new(provider.reference.clone(), lattice).await?);

  let service = NativeProviderService::new(provider);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(seed))]
pub(crate) fn initialize_native_provider(namespace: String, seed: Seed) -> ProviderInitResult {
  trace!("registering");
  let provider = Arc::new(vino_stdlib::Provider::new(seed));
  let service = NativeProviderService::new(provider);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}
