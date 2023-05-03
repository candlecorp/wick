pub(crate) mod component_service;
pub(crate) mod engine_component;
pub(crate) mod error;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flow_component::{Component, RuntimeCallback};
use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_component_wasm::error::LinkError;
use wick_config::config::components::{ManifestComponent, WasmComponent};
use wick_config::config::{BoundInterface, FetchOptions, Metadata, WasmComponentImplementation};
use wick_config::{Resolver, WickConfiguration};
use wick_packet::{Entity, Invocation, OperationConfig, PacketStream};

use self::component_service::NativeComponentService;
use crate::dev::prelude::*;
use crate::dispatch::engine_invoke_async;
use crate::runtime_service::ComponentInitOptions;
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(
    &self,
    msg: Invocation,
    stream: PacketStream,
    config: Option<OperationConfig>,
  ) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ComponentError>;

type ComponentInitResult = std::result::Result<NamespaceHandler, EngineError>;

pub(crate) async fn init_wasm_component<'a, 'b>(
  kind: &'a WasmComponent,
  namespace: String,
  opts: ComponentInitOptions<'b>,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  trace!(namespace = %namespace, ?opts, "registering wasm component");

  let component =
    wick_component_wasm::helpers::load_wasm(&kind.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  // TODO take max threads from configuration
  let collection = Arc::new(
    wick_component_wasm::component::WasmComponent::try_load(
      &component,
      5,
      Some(kind.permissions.clone()),
      None,
      Some(make_link_callback(opts.runtime_id)),
      provided,
    )
    .await?,
  );

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_wasmrs_component<'a, 'b>(
  kind: &'a WasmComponentImplementation,
  namespace: String,
  opts: ComponentInitOptions<'b>,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  trace!(namespace = %namespace, ?opts, "registering wasmrs component");

  let component =
    wick_component_wasm::helpers::load_wasm(kind.reference(), opts.allow_latest, &opts.allowed_insecure).await?;

  // TODO take max threads from configuration
  let collection = Arc::new(
    wick_component_wasm::component::WasmComponent::try_load(
      &component,
      5,
      None,
      None,
      Some(make_link_callback(opts.runtime_id)),
      provided,
    )
    .await?,
  );

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}
pub(crate) fn make_link_callback(engine_id: Uuid) -> Arc<RuntimeCallback> {
  Arc::new(move |compref, op, stream, inherent, config| {
    let origin_url = compref.get_origin_url();
    let target_id = compref.get_target_id().to_owned();
    Box::pin(async move {
      {
        debug!(
          origin = %origin_url,
          target = %target_id,
          engine_id = %engine_id,
          "link_call"
        );
        let invocation = compref.to_invocation(&op, inherent);

        // let target = Entity::from_str(&target_id).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        // let origin = Entity::from_str(&origin_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        // if let Entity::Operation(origin_ns, _) = &origin {
        //   if let Entity::Operation(target_ns, _) = &target {
        //     if target_ns == origin_ns {
        //       return Err(LinkError::Circular(target_ns.clone()));
        //     }
        //   }
        // }

        let result = engine_invoke_async(engine_id, invocation, stream, config)
          .await
          .map_err(|e| flow_component::ComponentError::new(LinkError::CallFailure(e.to_string())))?;
        Ok(result)
      }
    })
  })
}

pub(crate) async fn init_manifest_component<'a, 'b>(
  kind: &'a ManifestComponent,
  id: String,
  mut opts: ComponentInitOptions<'b>,
) -> ComponentInitResult {
  trace!(namespace = %id, ?opts, "registering composite component");

  let options = FetchOptions::new()
    .allow_latest(opts.allow_latest)
    .allow_insecure(&opts.allowed_insecure);
  let manifest = WickConfiguration::fetch(kind.reference.path()?.to_string_lossy(), options)
    .await?
    .try_component_config()?;

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();
  let requires = manifest.requires();
  let metadata = manifest.metadata();
  let provided =
    generate_provides(requires, &kind.provide).map_err(|e| EngineError::ComponentInit(id.clone(), e.to_string()))?;

  match manifest.component() {
    config::ComponentImplementation::Wasm(wasmimpl) => {
      let comp = init_wasmrs_component(wasmimpl, id.clone(), opts, provided).await?;
      let signed_sig = comp.component().list();
      let manifest_sig = manifest.signature()?;
      expect_signature_match(
        Some(&PathBuf::from(&id)),
        signed_sig,
        Some(&PathBuf::from(wasmimpl.reference().location())),
        &manifest_sig,
      )?;
      Ok(comp)
    }
    config::ComponentImplementation::Composite(_) => {
      let _engine = RuntimeService::new_from_manifest(uuid, manifest, Some(id.clone()), opts).await?;

      let collection = Arc::new(engine_component::EngineComponent::new(uuid));
      let service = NativeComponentService::new(collection);
      Ok(NamespaceHandler::new(id, Box::new(service)))
    }
    config::ComponentImplementation::Sql(c) => {
      init_hlc_component(
        id,
        metadata,
        wick_config::config::HighLevelComponent::Sql(c.clone()),
        manifest.resolver(),
      )
      .await
    }
    config::ComponentImplementation::HttpClient(c) => {
      init_hlc_component(
        id,
        metadata,
        wick_config::config::HighLevelComponent::HttpClient(c.clone()),
        manifest.resolver(),
      )
      .await
    }
  }
}

pub(crate) fn expect_signature_match(
  actual_src: Option<&Path>,
  actual: &ComponentSignature,
  expected_src: Option<&Path>,
  expected: &ComponentSignature,
) -> std::result::Result<(), EngineError> {
  if actual != expected {
    error!(
      // expected = serde_json::to_string(expected).unwrap(),
      ?expected,
      // actual = serde_json::to_string(actual).unwrap(),
      ?actual,
      "signature mismatch"
    );
    return Err(EngineError::ComponentSignature(
      expected_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
      actual_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
    ));
  }
  Ok(())
}

fn generate_provides(
  requires: &HashMap<String, BoundInterface>,
  provides: &HashMap<String, String>,
) -> Result<HashMap<String, String>> {
  let mut provide = HashMap::new();
  #[allow(clippy::for_kv_map)] // silencing clippy to keep context for the TODO below.
  for (id, _interface) in requires {
    if let Some(provided) = provides.get(id) {
      provide.insert(id.clone(), Entity::component(provided).url());
      // TODO: validate interfaces against what was provided.
    } else {
      return Err(ComponentError::UnsatisfiedRequirement(id.clone()));
    }
  }
  Ok(provide)
}

pub(crate) fn initialize_native_component(namespace: String, seed: Seed) -> ComponentInitResult {
  let collection = Arc::new(wick_stdlib::Collection::new(seed));
  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_hlc_component(
  id: String,
  metadata: Metadata,
  component: wick_config::config::HighLevelComponent,
  resolver: Box<Resolver>,
) -> ComponentInitResult {
  let comp: Box<dyn Component + Send + Sync> = match component {
    config::HighLevelComponent::Sql(config) => Box::new(wick_sqlx::SqlXComponent::new(config, metadata, &resolver)?),
    config::HighLevelComponent::HttpClient(config) => {
      Box::new(wick_http_client::HttpClientComponent::new(config, metadata, &resolver)?)
    }
  };
  comp.init().await.map_err(EngineError::NativeComponent)?;
  Ok(NamespaceHandler::new(id, comp))
}
