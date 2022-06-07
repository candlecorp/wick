pub(crate) mod collection_service;
pub(crate) mod error;
pub(crate) mod network_collection;

use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wasmflow_collection_wasm::collection::HostLinkCallback;
use wasmflow_collection_wasm::error::LinkError;
use wasmflow_interpreter::NamespaceHandler;
use wasmflow_manifest::network_definition::EntrypointDefinition;

use self::collection_service::NativeCollectionService;
use crate::dev::prelude::*;
use crate::dispatch::network_invoke_sync;
use crate::network_service::CollectionInitOptions;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<CollectionSignature>;
  fn invoke(&self, msg: Invocation) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, CollectionError>;

type CollectionInitResult = Result<NamespaceHandler>;

#[instrument(skip(collection, opts))]
pub(crate) async fn initialize_par_collection(
  collection: &CollectionDefinition,
  namespace: String,
  opts: CollectionInitOptions,
) -> CollectionInitResult {
  trace!(namespace = %collection.namespace, ?opts, "registering");

  let bytes = wasmflow_loader::get_bytes(&collection.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  let service = wasmflow_collection_par::collection::Collection::from_tarbytes(
    collection.reference.clone(),
    &*bytes,
    Some(collection.data.clone()),
  )
  .await?;

  let service = NativeCollectionService::new(Arc::new(service));

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(collection))]
pub(crate) async fn initialize_grpc_collection(
  collection: &CollectionDefinition,
  namespace: String,
) -> CollectionInitResult {
  trace!(namespace = %collection.namespace, "registering");

  let service = wasmflow_collection_grpc::collection::Collection::new(collection.reference.clone()).await?;

  let service = NativeCollectionService::new(Arc::new(service));

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(collection, opts))]
pub(crate) async fn initialize_wasm_collection(
  collection: &CollectionDefinition,
  namespace: String,
  opts: CollectionInitOptions,
) -> CollectionInitResult {
  trace!(namespace = %collection.namespace, ?opts, "registering");

  let component =
    wasmflow_collection_wasm::helpers::load_wasm(&collection.reference, opts.allow_latest, &opts.allowed_insecure)
      .await?;

  // TODO take max threads from configuration
  let collection = Arc::new(wasmflow_collection_wasm::collection::Collection::try_load(
    &component,
    5,
    Some(collection.data.clone()),
    None,
    Some(make_link_callback(opts.network_id)),
  )?);

  let service = NativeCollectionService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip_all)]
pub(crate) async fn initialize_wasm_entrypoint(
  entrypoint: &EntrypointDefinition,
  network_id: Uuid,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<wasmflow_collection_wasm::collection::Collection> {
  trace!(%network_id, "registering entrypoint");

  let component =
    wasmflow_collection_wasm::helpers::load_wasm(&entrypoint.reference, allow_latest, allowed_insecure).await?;

  // TODO take max threads from configuration
  let collection = wasmflow_collection_wasm::collection::Collection::try_load(
    &component,
    1,
    Some(entrypoint.data.clone()),
    None,
    Some(make_link_callback(network_id)),
  )?;

  Ok(collection)
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
    let invocation = Invocation::new(origin, target, payload.into(), None);
    let result = network_invoke_sync(network_id, invocation).map_err(|e| LinkError::CallFailure(e.to_string()))?;
    Ok(result.into_iter().map(|v| v.into()).collect())
  })
}

#[instrument(skip(collection, opts))]
pub(crate) async fn initialize_network_collection(
  collection: &CollectionDefinition,
  namespace: String,
  mut opts: CollectionInitOptions,
) -> CollectionInitResult {
  trace!(namespace = %collection.namespace, ?opts, "registering");

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();

  let _network = NetworkService::new_from_manifest(uuid, &collection.reference, Some(namespace.clone()), opts)
    .await
    .map_err(|e| CollectionError::SubNetwork(collection.reference.clone(), e.to_string()))?;

  let collection = Arc::new(network_collection::Collection::new(uuid));

  let service = NativeCollectionService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(collection, opts))]
pub(crate) async fn initialize_mesh_collection(
  collection: &CollectionDefinition,
  namespace: String,
  opts: CollectionInitOptions,
) -> CollectionInitResult {
  trace!(namespace = %collection.namespace, ?opts, "registering");
  let mesh = match opts.mesh {
    Some(mesh) => mesh,
    None => {
      return Err(CollectionError::Mesh(
        "Mesh collection defined but no mesh available".to_owned(),
      ))
    }
  };

  let collection =
    Arc::new(wasmflow_collection_nats::collection::Collection::new(collection.reference.clone(), mesh).await?);

  let service = NativeCollectionService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

#[instrument(skip(seed))]
pub(crate) fn initialize_native_collection(namespace: String, seed: Seed) -> CollectionInitResult {
  trace!("registering");
  let collection = Arc::new(wasmflow_stdlib::Collection::new(seed));
  let service = NativeCollectionService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}
