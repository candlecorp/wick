use wick_asset_reference::AssetReference;

use crate::config::ComponentOperationExpression;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
pub struct RestRouterConfig {
  /// The path to start serving this router from.
  #[asset(skip)]
  pub path: String,
  /// Additional tools and services to enable.
  #[asset(skip)]
  pub tools: Option<Tools>,
  /// The routes to serve and operations that handle them.
  pub routes: Vec<Route>,
  /// Information about the router to use when generating documentation and other tools.
  #[asset(skip)]
  pub info: Option<Info>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tools {
  /// The path to serve the OpenAPI spec from
  pub openapi: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
/// Information about the router to use when generating documentation and other tools.
pub struct Info {
  /// The title of the API.
  pub title: Option<String>,
  /// A short description of the API.
  pub description: Option<String>,
  /// The terms of service for the API.
  pub tos: Option<String>,
  /// The contact information for the API.
  pub contact: Option<Contact>,
  /// The license information for the API.
  pub license: Option<License>,
  /// The version of the API.
  pub version: Option<String>,
  /// The URL to the API&#x27;s terms of service.
  pub documentation: Option<Documentation>,
}

#[derive(Debug, Default, Clone, PartialEq)]
/// Documentation information for the API.
pub struct Documentation {
  /// The URL to the API&#x27;s documentation.
  pub url: Option<String>,
  /// A short description of the documentation.
  pub description: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
/// The license information for the API.
pub struct License {
  /// The name of the license.
  pub name: Option<String>,
  /// The URL to the license.
  pub url: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
/// The contact information for the API.
pub struct Contact {
  /// The name of the contact.
  pub name: Option<String>,
  /// The URL to the contact.
  pub url: Option<String>,
  /// The email address of the contact.
  pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
/// A route to serve and the operation that handles it.
pub struct Route {
  /// The name of the route, used for documentation and tooling.
  #[asset(skip)]
  pub name: Option<String>,
  /// The HTTP methods to serve this route for.
  #[asset(skip)]
  pub methods: Vec<String>,
  /// The path to serve this route from.
  #[asset(skip)]
  pub uri: String,
  /// The operation that will act as the main entrypoint for this route.
  pub operation: ComponentOperationExpression,
  /// A short description of the route.
  #[asset(skip)]
  pub description: Option<String>,
  /// A longer description of the route.
  #[asset(skip)]
  pub summary: Option<String>,
}
