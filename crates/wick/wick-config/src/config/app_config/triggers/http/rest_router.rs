use wick_asset_reference::AssetReference;

use crate::config::common::HttpMethod;
use crate::config::ComponentOperationExpression;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
pub struct RestRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  #[property(get(disable))]
  pub(crate) path: String,
  /// Middleware operations for this router.
  #[property(get(disable))]
  pub(crate) middleware: Option<super::middleware::Middleware>,
  /// Additional tools and services to enable.
  #[asset(skip)]
  pub(crate) tools: Option<Tools>,
  /// The routes to serve and operations that handle them.
  pub(crate) routes: Vec<RestRoute>,
  /// Information about the router to use when generating documentation and other tools.
  #[asset(skip)]
  pub(crate) info: Option<Info>,
}

impl super::WickRouter for RestRouterConfig {
  fn middleware(&self) -> Option<&super::Middleware> {
    self.middleware.as_ref()
  }

  fn path(&self) -> &str {
    &self.path
  }
}

#[derive(Debug, Default, Clone, PartialEq, property::Property)]
#[property(get(public), set(disable), mut(disable))]
#[allow(missing_copy_implementations)]
pub struct Tools {
  /// Set to true to generate an OpenAPI specification and serve it at *router_path*/openapi.json
  pub(crate) openapi: bool,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// Information about the router to use when generating documentation and other tools.
pub struct Info {
  /// The title of the API.
  pub(crate) title: Option<String>,
  /// A short description of the API.
  pub(crate) description: Option<String>,
  /// The terms of service for the API.
  pub(crate) tos: Option<String>,
  /// The contact information for the API.
  pub(crate) contact: Option<Contact>,
  /// The license information for the API.
  pub(crate) license: Option<License>,
  /// The version of the API.
  pub(crate) version: String,
  /// The URL to the API&#x27;s terms of service.
  pub(crate) documentation: Option<Documentation>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// Documentation information for the API.
pub struct Documentation {
  /// The URL to the API&#x27;s documentation.
  pub(crate) url: Option<String>,
  /// A short description of the documentation.
  pub(crate) description: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// The license information for the API.
pub struct License {
  /// The name of the license.
  pub(crate) name: String,
  /// The URL to the license.
  pub(crate) url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// The contact information for the API.
pub struct Contact {
  /// The name of the contact.
  pub(crate) name: Option<String>,
  /// The URL to the contact.
  pub(crate) url: Option<String>,
  /// The email address of the contact.
  pub(crate) email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]
/// A route to serve and the operation that handles it.
pub struct RestRoute {
  /// The name of the route, used for documentation and tooling.
  #[asset(skip)]
  pub(crate) id: Option<String>,
  /// The HTTP methods to serve this route for.
  #[asset(skip)]
  pub(crate) methods: Vec<HttpMethod>,
  /// The path to serve this route from.
  #[asset(skip)]
  pub(crate) sub_path: String,
  /// The operation that will act as the main entrypoint for this route.
  pub(crate) operation: ComponentOperationExpression,
  /// A short description of the route.
  #[asset(skip)]
  pub(crate) description: Option<String>,
  /// A longer description of the route.
  #[asset(skip)]
  pub(crate) summary: Option<String>,
}
