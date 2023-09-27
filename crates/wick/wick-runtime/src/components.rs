pub(crate) mod component_service;
pub(crate) mod error;
pub(crate) mod scope_component;
pub(crate) mod validation;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use flow_component::LocalScope;
use flow_graph_interpreter::NamespaceHandler;
use seeded_random::Random;
use tracing::Instrument;
use uuid::Uuid;
use wick_component_wasmrs::component::WasmrsComponent;
use wick_component_wasmrs::error::LinkError;
use wick_config::config::components::ManifestComponent;
use wick_config::config::{Metadata, Permissions, PermissionsBuilder, WasmComponentDefinition, WasmRsComponent};
use wick_config::{AssetReference, FetchOptions, Resolver, WickConfiguration};
use wick_packet::validation::expect_configuration_matches;
use wick_packet::{Entity, Invocation, RuntimeConfig};

use self::component_service::NativeComponentService;
use self::validation::expect_signature_match;
use crate::dev::prelude::*;
use crate::dispatch::scope_invoke_async;
use crate::runtime::scope::{init_child, ChildInit};
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(&self, msg: Invocation, config: Option<RuntimeConfig>) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ComponentError>;

type ComponentInitResult = std::result::Result<NamespaceHandler, ScopeError>;

pub(crate) async fn init_wasmrs_component(
  reference: &AssetReference,
  namespace: String,
  opts: ChildInit,
  buffer_size: Option<u32>,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
  imported: HashMap<String, String>,
) -> ComponentInitResult {
  opts
    .span
    .in_scope(|| trace!(namespace = %namespace, ?opts, ?permissions, "registering wasmrs component"));

  let mut options = FetchOptions::default();
  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());
  let asset = reference.with_options(options);

  use wick_component_wasmrs::component::ComponentSetupBuilder;

  let setup = ComponentSetupBuilder::default()
    .buffer_size(buffer_size)
    .permissions(permissions)
    .config(opts.root_config)
    .callback(Some(make_link_callback(opts.runtime_id)))
    .provided(provided)
    .imported(imported)
    .build()
    .unwrap();

  let component = WasmrsComponent::try_load(&namespace, asset, setup, opts.span).await?;

  let component = Arc::new(component);

  let service = NativeComponentService::new(component);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_wasm_component(
  reference: &AssetReference,
  namespace: String,
  opts: ChildInit,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
  imported: HashMap<String, String>,
) -> ComponentInitResult {
  opts
    .span
    .in_scope(|| trace!(namespace = %namespace, ?opts, ?permissions, "registering wasmrs component"));

  let mut options = FetchOptions::default();
  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());
  let asset = reference.with_options(options);

  use wick_component_wasm::component::{ComponentSetupBuilder, WasmComponent};

  let setup = ComponentSetupBuilder::default()
    .permissions(permissions)
    .config(opts.root_config)
    .callback(make_link_callback(opts.runtime_id))
    .provided(provided)
    .imported(imported)
    .build()
    .unwrap();

  let component = WasmComponent::try_load(&namespace, asset, setup, opts.span).await?;

  let component = Arc::new(component);

  let service = NativeComponentService::new(component);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_wasmrs_def(
  kind: &WasmRsComponent,
  namespace: String,
  opts: ChildInit,
  buffer_size: Option<u32>,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
  imported: HashMap<String, String>,
) -> ComponentInitResult {
  init_wasmrs_component(
    kind.reference(),
    namespace,
    opts,
    buffer_size.or(kind.max_packet_size()),
    permissions,
    provided,
    imported,
  )
  .await
}

pub(crate) async fn init_wasm_def(
  kind: &WasmComponentDefinition,
  namespace: String,
  opts: ChildInit,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
  imported: HashMap<String, String>,
) -> ComponentInitResult {
  init_wasm_component(kind.reference(), namespace, opts, permissions, provided, imported).await
}

pub(crate) fn make_link_callback(scope_id: Uuid) -> LocalScope {
  LocalScope::new(Arc::new(move |compref, op, stream, inherent, config, span| {
    let origin_url = compref.get_origin_url();
    let target_id = compref.get_target_id().to_owned();
    let invocation = compref.to_invocation(&op, stream, inherent, span);
    invocation.trace(|| {
      debug!(
        origin = %origin_url,
        target = %target_id,
        scope_id = %scope_id,
        config = ?config,
        "link_call"
      );
    });
    Box::pin(async move {
      {
        let result = scope_invoke_async(scope_id, invocation, config)
          .await
          .map_err(|e| flow_component::ComponentError::new(LinkError::CallFailure(e.to_string())))?;
        Ok(result)
      }
    })
  }))
}

pub(crate) async fn init_manifest_component(
  kind: &ManifestComponent,
  id: String,
  mut opts: ChildInit,
) -> ComponentInitResult {
  let span = opts.span.clone();
  span.in_scope(|| trace!(namespace = %id, ?opts, "registering wick component"));

  let mut options = FetchOptions::default();

  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());
  let mut builder = WickConfiguration::fetch(kind.reference().clone(), options)
    .instrument(span.clone())
    .await?;

  builder.set_root_config(opts.root_config.clone());
  let manifest = builder.finish()?.try_component_config()?;

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();

  let uuid = rng.uuid();
  let _scope = init_child(uuid, manifest.clone(), id.clone(), opts).await?;

  let component = Arc::new(scope_component::ScopeComponent::new(uuid));
  let service = NativeComponentService::new(component);
  Ok(NamespaceHandler::new(id, Box::new(service)))
}

pub(crate) async fn init_impl(
  manifest: &ComponentConfiguration,
  id: String,
  mut opts: ChildInit,
  buffer_size: Option<u32>,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  let span = opts.span.clone();
  span.in_scope(|| {
    debug!(%id,"validating configuration for wick component");
    expect_configuration_matches(&id, opts.root_config.as_ref(), manifest.config()).map_err(ScopeError::Setup)
  })?;

  let resolver = manifest.resolver();

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let metadata = manifest.metadata();
  match manifest.component() {
    config::ComponentImplementation::Wasm(wasmimpl) => {
      let mut dirs = HashMap::new();
      for volume in wasmimpl.volumes() {
        let resource = (resolver)(volume.resource())?.try_resource()?.try_volume()?;
        dirs.insert(volume.path().to_owned(), resource.path()?);
      }
      let perms = (!dirs.is_empty()).then(|| PermissionsBuilder::default().dirs(dirs).build().unwrap());

      let imported: HashMap<String, String> = manifest
        .import()
        .iter()
        .map(|i| (i.id().to_owned(), Entity::component(i.id()).url()))
        .collect();
      let comp = init_wasm_def(wasmimpl, id.clone(), opts, perms, provided, imported).await?;
      let signed_sig = comp.component().signature();
      let manifest_sig = manifest.signature()?;
      span.in_scope(|| {
        expect_signature_match(
          Some(&PathBuf::from(&id)),
          signed_sig,
          Some(&PathBuf::from(wasmimpl.reference().location())),
          &manifest_sig,
        )
      })?;
      Ok(comp)
    }
    config::ComponentImplementation::WasmRs(wasmimpl) => {
      let mut dirs = HashMap::new();
      for volume in wasmimpl.volumes() {
        let resource = (resolver)(volume.resource())?.try_resource()?.try_volume()?;
        dirs.insert(volume.path().to_owned(), resource.path()?);
      }
      let perms = (!dirs.is_empty()).then(|| PermissionsBuilder::default().dirs(dirs).build().unwrap());

      let imported: HashMap<String, String> = manifest
        .import()
        .iter()
        .map(|i| (i.id().to_owned(), Entity::component(i.id()).url()))
        .collect();
      let comp = init_wasmrs_def(wasmimpl, id.clone(), opts, buffer_size, perms, provided, imported).await?;
      let signed_sig = comp.component().signature();
      let manifest_sig = manifest.signature()?;
      span.in_scope(|| {
        expect_signature_match(
          Some(&PathBuf::from(&id)),
          signed_sig,
          Some(&PathBuf::from(wasmimpl.reference().location())),
          &manifest_sig,
        )
      })?;
      Ok(comp)
    }
    config::ComponentImplementation::Composite(_) => {
      // This is handled in the scope initialization.
      unreachable!();
    }
    config::ComponentImplementation::Sql(c) => {
      init_hlc_component(
        id,
        opts.root_config.clone(),
        metadata.cloned(),
        wick_config::config::HighLevelComponent::Sql(c.clone()),
        manifest.resolver(),
      )
      .await
    }
    config::ComponentImplementation::HttpClient(c) => {
      init_hlc_component(
        id,
        opts.root_config.clone(),
        metadata.cloned(),
        wick_config::config::HighLevelComponent::HttpClient(c.clone()),
        manifest.resolver(),
      )
      .await
    }
  }
}

pub(crate) async fn init_hlc_component(
  id: String,
  root_config: Option<RuntimeConfig>,
  metadata: Option<Metadata>,
  component: wick_config::config::HighLevelComponent,
  resolver: Box<Resolver>,
) -> ComponentInitResult {
  let comp: Box<dyn Component + Send + Sync> = match component {
    config::HighLevelComponent::Sql(comp) => {
      Box::new(wick_sql::SqlComponent::new(comp, root_config, metadata, &resolver).await?)
    }
    config::HighLevelComponent::HttpClient(comp) => Box::new(wick_http_client::HttpClientComponent::new(
      comp,
      root_config,
      metadata,
      &resolver,
    )?),
  };
  Ok(NamespaceHandler::new(id, comp))
}
