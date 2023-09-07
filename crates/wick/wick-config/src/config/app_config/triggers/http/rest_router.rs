use std::collections::HashMap;
use std::path::Path;

use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use super::index_to_router_id;
use super::middleware::expand_for_middleware_components;
use crate::config::common::HttpMethod;
use crate::config::template_config::Renderable;
use crate::config::{self, Binding, ComponentOperationExpression, ImportDefinition};
use crate::error::ManifestError;

#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_builder::Builder,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
pub struct RestRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable), mut(disable))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  /// Additional tools and services to enable.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) tools: Option<Tools>,
  /// The routes to serve and operations that handle them.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) routes: Vec<RestRoute>,
  /// Information about the router to use when generating documentation and other tools.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) info: Option<Info>,
}

impl Renderable for RestRouterConfig {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.middleware.render_config(source, root_config, env)?;
    self.routes.render_config(source, root_config, env)
  }
}

impl Renderable for RestRoute {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    self.operation.render_config(source, root_config, env)
  }
}

impl super::WickRouter for RestRouterConfig {
  fn middleware(&self) -> Option<&super::Middleware> {
    self.middleware.as_ref()
  }
  fn middleware_mut(&mut self) -> Option<&mut super::Middleware> {
    self.middleware.as_mut()
  }

  fn path(&self) -> &str {
    &self.path
  }
}

#[derive(Debug, Default, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(disable), mut(disable))]
#[allow(missing_copy_implementations)]
pub struct Tools {
  /// Set to true to generate an OpenAPI specification and serve it at *router_path*/openapi.json
  pub(crate) openapi: bool,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// Information about the router to use when generating documentation and other tools.
pub struct Info {
  /// The title of the API.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) title: Option<String>,
  /// A short description of the API.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// The terms of service for the API.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) tos: Option<String>,
  /// The contact information for the API.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) contact: Option<Contact>,
  /// The license information for the API.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) license: Option<License>,
  /// The version of the API.
  pub(crate) version: String,
  /// The URL to the API&#x27;s terms of service.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) documentation: Option<Documentation>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// Documentation information for the API.
pub struct Documentation {
  /// The URL to the API&#x27;s documentation.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
  /// A short description of the documentation.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// The license information for the API.
pub struct License {
  /// The name of the license.
  pub(crate) name: String,
  /// The URL to the license.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// The contact information for the API.
pub struct Contact {
  /// The name of the contact.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// The URL to the contact.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
  /// The email address of the contact.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) email: Option<String>,
}

#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_builder::Builder,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
/// A route to serve and the operation that handles it.
pub struct RestRoute {
  /// The name of the route, used for documentation and tooling.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) id: Option<String>,
  /// The HTTP methods to serve this route for.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) methods: Vec<HttpMethod>,
  /// The path to serve this route from.
  #[asset(skip)]
  pub(crate) sub_path: String,
  /// The operation that will act as the main entrypoint for this route.
  pub(crate) operation: ComponentOperationExpression,
  /// A short description of the route.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// A longer description of the route.
  #[asset(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) summary: Option<String>,
}

pub(crate) fn process_runtime_config(
  trigger_index: usize,
  index: usize,
  router_config: &mut RestRouterConfig,
  bindings: &mut Vec<Binding<ImportDefinition>>,
) -> Result<(), ManifestError> {
  expand_for_middleware_components(trigger_index, index, router_config, bindings)?;

  for (i, route) in router_config.routes_mut().iter_mut().enumerate() {
    let component_id = format!("{}_{}", index_to_router_id(trigger_index, index), i);
    route.operation_mut().maybe_import(&component_id, bindings);
  }

  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::Binding::new(
    index_to_router_id(trigger_index, index),
    ImportDefinition::component(router_component),
  );
  bindings.push(router_binding);
  Ok(())
}
