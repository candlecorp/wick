pub(crate) mod component_service;
pub(crate) mod engine_component;
pub(crate) mod error;

use std::collections::HashMap;
use std::sync::Arc;

use flow_component::RuntimeCallback;
use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_component_wasm::error::LinkError;
use wick_config::config::components::{ManifestComponent, WasmComponent};
use wick_config::config::{BoundInterface, FetchOptions};
use wick_config::WickConfiguration;
use wick_packet::{Entity, Invocation, PacketStream};

use self::component_service::NativeComponentService;
use crate::dev::prelude::*;
use crate::dispatch::engine_invoke_async;
use crate::runtime_service::ComponentInitOptions;
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(&self, msg: Invocation, stream: PacketStream) -> Result<BoxFuture<Result<InvocationResponse>>>;
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

pub(crate) fn make_link_callback(engine_id: Uuid) -> Arc<RuntimeCallback> {
  Arc::new(move |compref, op, stream, inherent| {
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
        let invocation = compref.make_invocation(&op, inherent);

        // let target = Entity::from_str(&target_id).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        // let origin = Entity::from_str(&origin_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        // if let Entity::Operation(origin_ns, _) = &origin {
        //   if let Entity::Operation(target_ns, _) = &target {
        //     if target_ns == origin_ns {
        //       return Err(LinkError::Circular(target_ns.clone()));
        //     }
        //   }
        // }

        let result = engine_invoke_async(engine_id, invocation, stream)
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
  let manifest = WickConfiguration::fetch(kind.reference.path()?, options)
    .await?
    .try_component_config()?;

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();

  match manifest.component() {
    config::ComponentImplementation::Wasm(wasmimpl) => {
      let provided = generate_provides(wasmimpl.requires(), &kind.provide)
        .map_err(|e| EngineError::ComponentInit(id.clone(), e.to_string()))?;

      let wasm = WasmComponent {
        reference: wasmimpl.reference().clone(),
        config: Default::default(),
        permissions: Default::default(),
        provide: Default::default(),
      };
      let comp = init_wasm_component(&wasm, id.clone(), opts, provided).await?;
      let signed_sig = comp.component().list();
      let manifest_sig = manifest.signature()?;
      expect_signature_match(&id, signed_sig, wasmimpl.reference().location(), &manifest_sig)?;
      Ok(comp)
    }
    config::ComponentImplementation::Composite(composite) => {
      let _provide = generate_provides(composite.requires(), &kind.provide)
        .map_err(|e| EngineError::ComponentInit(id.clone(), e.to_string()))?;

      let _engine = RuntimeService::new_from_manifest(uuid, manifest, Some(id.clone()), opts).await?;

      let collection = Arc::new(engine_component::EngineComponent::new(uuid));
      let service = NativeComponentService::new(collection);
      Ok(NamespaceHandler::new(id, Box::new(service)))
    }
  }
}

pub(crate) fn expect_signature_match(
  actual_src: impl AsRef<str>,
  actual: &ComponentSignature,
  expected_src: impl AsRef<str>,
  expected: &ComponentSignature,
) -> std::result::Result<(), EngineError> {
  if actual != expected {
    error!(
      expected = serde_json::to_string(expected).unwrap(),
      actual = serde_json::to_string(actual).unwrap(),
      "signature mismatch"
    );
    return Err(EngineError::ComponentSignature(
      expected_src.as_ref().to_owned(),
      actual_src.as_ref().to_owned(),
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
