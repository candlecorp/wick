pub(crate) mod component_service;
pub(crate) mod engine_component;
pub(crate) mod error;
pub(crate) mod validation;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use flow_component::{Component, RuntimeCallback};
use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use tracing::Instrument;
use uuid::Uuid;
use wick_component_wasm::error::LinkError;
use wick_config::config::components::{ManifestComponent, Permissions};
use wick_config::config::{
  AssetReference,
  BoundInterface,
  FetchOptions,
  Metadata,
  ResourceDefinition,
  WasmComponentImplementation,
};
use wick_config::{Resolver, WickConfiguration};
use wick_packet::validation::expect_configuration_matches;
use wick_packet::{Entity, GenericConfig, Invocation};

use self::component_service::NativeComponentService;
use self::validation::expect_signature_match;
use crate::dev::prelude::*;
use crate::dispatch::engine_invoke_async;
use crate::runtime_service::ComponentInitOptions;
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(&self, msg: Invocation, config: Option<GenericConfig>) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ComponentError>;

type ComponentInitResult = std::result::Result<NamespaceHandler, EngineError>;

pub(crate) async fn init_wasm_component(
  reference: &AssetReference,
  permissions: Option<Permissions>,
  namespace: String,
  opts: ComponentInitOptions,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  opts
    .span
    .in_scope(|| trace!(namespace = %namespace, ?opts, "registering wasm component"));

  let component = wick_component_wasm::helpers::fetch_wasm(reference, opts.allow_latest, &opts.allowed_insecure)
    .instrument(opts.span.clone())
    .await?;

  // TODO take max threads from configuration
  let collection = Arc::new(
    wick_component_wasm::component::WasmComponent::try_load(
      &component,
      5,
      permissions,
      opts.config,
      Some(make_link_callback(opts.runtime_id)),
      provided,
      opts.span,
    )
    .await?,
  );

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_wasm_impl_component(
  kind: &WasmComponentImplementation,
  namespace: String,
  opts: ComponentInitOptions,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  init_wasm_component(kind.reference(), None, namespace, opts, provided).await
}

pub(crate) fn make_link_callback(engine_id: Uuid) -> Arc<RuntimeCallback> {
  Arc::new(move |compref, op, stream, inherent, config, span| {
    let origin_url = compref.get_origin_url();
    let target_id = compref.get_target_id().to_owned();
    let invocation = compref.to_invocation(&op, stream, inherent, span);
    invocation.trace(|| {
      debug!(
        origin = %origin_url,
        target = %target_id,
        engine_id = %engine_id,
        config = ?config,
        "link_call"
      );
    });
    Box::pin(async move {
      {
        let result = engine_invoke_async(engine_id, invocation, config)
          .await
          .map_err(|e| flow_component::ComponentError::new(LinkError::CallFailure(e.to_string())))?;
        Ok(result)
      }
    })
  })
}

pub(crate) async fn init_manifest_component(
  kind: &ManifestComponent,
  id: String,
  mut opts: ComponentInitOptions,
) -> ComponentInitResult {
  opts
    .span
    .in_scope(|| trace!(namespace = %id, ?opts, "registering composite component"));

  let mut options = FetchOptions::default();

  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());

  let manifest = WickConfiguration::fetch(kind.reference().path()?.to_string_lossy(), options)
    .instrument(opts.span.clone())
    .await?
    .try_component_config()?;

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();
  let requires = manifest.requires();
  let metadata = manifest.metadata();
  let provided =
    generate_provides(requires, kind.provide()).map_err(|e| EngineError::ComponentInit(id.clone(), e.to_string()))?;

  expect_configuration_matches(&id, opts.config.as_ref(), manifest.component().config()).map_err(EngineError::Setup)?;

  match manifest.component() {
    config::ComponentImplementation::Wasm(wasmimpl) => {
      let comp = init_wasm_impl_component(wasmimpl, id.clone(), opts, provided).await?;
      let signed_sig = comp.component().signature();
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
      let _engine = RuntimeService::init_child(uuid, manifest, Some(id.clone()), opts).await?;

      let component = Arc::new(engine_component::EngineComponent::new(uuid));
      let service = NativeComponentService::new(component);
      Ok(NamespaceHandler::new(id, Box::new(service)))
    }
    config::ComponentImplementation::Sql(c) => {
      init_hlc_component(
        id,
        metadata.cloned(),
        wick_config::config::HighLevelComponent::Sql(c.clone()),
        manifest.resolver(),
      )
      .await
    }
    config::ComponentImplementation::HttpClient(c) => {
      init_hlc_component(
        id,
        metadata.cloned(),
        wick_config::config::HighLevelComponent::HttpClient(c.clone()),
        manifest.resolver(),
      )
      .await
    }
  }
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
  metadata: Option<Metadata>,
  component: wick_config::config::HighLevelComponent,
  resolver: Box<Resolver>,
) -> ComponentInitResult {
  let comp: Box<dyn Component + Send + Sync> = match component {
    config::HighLevelComponent::Sql(config) => {
      let url = resolver(config.resource())
        .ok_or_else(|| EngineError::ComponentInit("sql or azure-sql".to_owned(), "no resource found".to_owned()))?
        .try_resource()
        .unwrap();
      let scheme = match url {
        ResourceDefinition::Url(url) => url.scheme().to_owned(),
        _ => {
          return Err(EngineError::ComponentInit(
            "sql or azure-sql".to_owned(),
            "no resource found".to_owned(),
          ))
        }
      };
      if scheme == "mssql" {
        Box::new(wick_azure_sql::AzureSqlComponent::new(config, metadata, &resolver)?)
      } else {
        Box::new(wick_sqlx::SqlXComponent::new(config, metadata, &resolver)?)
      }
    }
    config::HighLevelComponent::HttpClient(config) => {
      Box::new(wick_http_client::HttpClientComponent::new(config, metadata, &resolver)?)
    }
  };
  comp.init().await.map_err(EngineError::NativeComponent)?;
  Ok(NamespaceHandler::new(id, comp))
}
