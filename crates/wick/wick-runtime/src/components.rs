pub(crate) mod component_service;
pub(crate) mod engine_component;
pub(crate) mod error;

use std::str::FromStr;
use std::sync::Arc;

use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_component_wasm::component::HostLinkCallback;
use wick_component_wasm::error::LinkError;
use wick_config::config::components::{ManifestComponent, WasmComponent};
use wick_config::config::FetchOptions;
use wick_config::WickConfiguration;
use wick_packet::{Entity, Invocation, PacketStream};

use self::component_service::NativeComponentService;
use crate::dev::prelude::*;
use crate::dispatch::engine_invoke_async;
use crate::engine_service::ComponentInitOptions;
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
) -> ComponentInitResult {
  trace!(namespace = %namespace, ?opts, "registering wasm component");

  let component =
    wick_component_wasm::helpers::load_wasm(&kind.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  // TODO take max threads from configuration
  let collection = Arc::new(wick_component_wasm::component::Component::try_load(
    &component,
    5,
    Some(kind.permissions.clone()),
    None,
    Some(make_link_callback(opts.engine_id)),
  )?);

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

fn make_link_callback(engine_id: Uuid) -> Box<HostLinkCallback> {
  Box::new(move |origin_url, target_url, stream| {
    let origin_url = origin_url.to_owned();
    let target_url = target_url.to_owned();
    Box::pin(async move {
      {
        debug!(
          origin = %origin_url,
          target = %target_url,
          engine_id = %engine_id,
          "link_call"
        );

        let target = Entity::from_str(&target_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        let origin = Entity::from_str(&origin_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        if let Entity::Operation(origin_ns, _) = &origin {
          if let Entity::Operation(target_ns, _) = &target {
            if target_ns == origin_ns {
              return Err(LinkError::Circular(target_ns.clone()));
            }
          }
        }

        let invocation = Invocation::new(origin, target, None);
        let result = engine_invoke_async(engine_id, invocation, stream)
          .await
          .map_err(|e| LinkError::CallFailure(e.to_string()))?;
        Ok(result)
      }
    })
  })
}

pub(crate) async fn init_manifest_component<'a, 'b>(
  kind: &'a ManifestComponent,
  namespace: String,
  mut opts: ComponentInitOptions<'b>,
) -> ComponentInitResult {
  trace!(namespace = %namespace, ?opts, "registering composite component");

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
    config::ComponentImplementation::Wasm(wasm) => {
      let wasm = WasmComponent {
        reference: wasm.reference().clone(),
        config: Default::default(),
        permissions: Default::default(),
      };
      init_wasm_component(&wasm, namespace, opts).await
    }
    config::ComponentImplementation::Composite(_) => {
      let _engine = EngineService::new_from_manifest(uuid, manifest, Some(namespace.clone()), opts).await?;

      let collection = Arc::new(engine_component::Component::new(uuid));
      let service = NativeComponentService::new(collection);
      Ok(NamespaceHandler::new(namespace, Box::new(service)))
    }
  }
}

pub(crate) fn initialize_native_component(namespace: String, seed: Seed, _span: &tracing::Span) -> ComponentInitResult {
  trace!("registering");
  let collection = Arc::new(wick_stdlib::Collection::new(seed));
  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}
