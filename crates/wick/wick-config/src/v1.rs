#![deny(
  warnings,
  missing_debug_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unreachable_pub,
  type_alias_bounds,
  trivial_bounds,
  mutable_transmutes,
  invalid_value,
  explicit_outlives_requirements,
  deprecated,
  clashing_extern_declarations,
  clippy::expect_used,
  clippy::explicit_deref_methods,
  missing_docs
)]
#![warn(clippy::cognitive_complexity)]
#![allow(clippy::large_enum_variant, missing_copy_implementations, clippy::enum_variant_names)]

pub(crate) mod conversions;
pub(crate) mod helpers;
pub(crate) mod impls;
pub(crate) mod parse;

use std::collections::HashMap;

use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with_expand_env::with_expand_envs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Root configuration that can be any one of the possible Wick configuration formats.
pub(crate) enum WickConfig {
  /// A variant representing a [AppConfiguration] type.
  #[serde(rename = "wick/app@v1")]
  AppConfiguration(AppConfiguration),
  /// A variant representing a [ComponentConfiguration] type.
  #[serde(rename = "wick/component@v1")]
  ComponentConfiguration(ComponentConfiguration),
  /// A variant representing a [TypesConfiguration] type.
  #[serde(rename = "wick/types@v1")]
  TypesConfiguration(TypesConfiguration),
  /// A variant representing a [TestConfiguration] type.
  #[serde(rename = "wick/tests@v1")]
  TestConfiguration(TestConfiguration),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for a standalone Wick application.
pub(crate) struct AppConfiguration {
  /// The application&#x27;s name.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Associated metadata for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<Metadata>,
  /// Details about the package for this application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageDefinition>,
  /// Resources and configuration that the application and its components can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,
  /// Components that to import and make available to the application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ImportBinding>,
  /// Triggers to load and instantiate to drive the application&#x27;s behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) triggers: Vec<TriggerDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Metadata to associate with an artifact.
pub(crate) struct Metadata {
  /// The version of the artifact.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) version: String,
  /// A list of the authors.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) authors: Vec<String>,
  /// A list of any vendors associated with the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) vendors: Vec<String>,
  /// A short description.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// Where to find documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) documentation: Option<String>,
  /// The license(s) for the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) licenses: Vec<String>,
  /// An icon to associate with the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) icon: Option<crate::v1::helpers::LocationReference>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for packaging and publishing Wick configurations.
pub(crate) struct PackageDefinition {
  /// The list of files and folders to be included with the package.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) files: Vec<crate::v1::helpers::Glob>,
  /// Configuration for publishing the package to a registry.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) registry: Option<RegistryDefinition>,
}

#[allow(non_snake_case)]
pub(crate) fn REGISTRY_DEFINITION_HOST() -> String {
  "registry.candle.dev".to_owned()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryDefinition {
  /// The registry to publish to, e.g. registry.candle.dev

  #[serde(default = "REGISTRY_DEFINITION_HOST")]
  #[serde(alias = "registry")]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) host: String,
  /// The namespace on the registry. e.g.: [*your username*]

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a resource.
pub(crate) struct ResourceBinding {
  /// The name of the binding.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The resource to bind to.
  pub(crate) resource: ResourceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to an imported component or type manifest.
pub(crate) struct ImportBinding {
  /// The name of the binding.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The import to bind to.
  pub(crate) component: ImportDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The possible types of resources. Resources are system-level resources and sensitive configuration.
pub(crate) enum ResourceDefinition {
  /// A variant representing a [TcpPort] type.
  #[serde(rename = "wick/resource/tcpport@v1")]
  TcpPort(TcpPort),
  /// A variant representing a [UdpPort] type.
  #[serde(rename = "wick/resource/udpport@v1")]
  UdpPort(UdpPort),
  /// A variant representing a [Url] type.
  #[serde(rename = "wick/resource/url@v1")]
  Url(Url),
  /// A variant representing a [Volume] type.
  #[serde(rename = "wick/resource/volume@v1")]
  Volume(Volume),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A TCP port to bind to.
pub(crate) struct TcpPort {
  /// The port to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: u16,
  /// The address to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A UDP port to bind to.
pub(crate) struct UdpPort {
  /// The port to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: u16,
  /// The address to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A filesystem or network volume resource.
pub(crate) struct Volume {
  /// The path.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A URL configured as a resource.
pub(crate) struct Url {
  /// The url string.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Triggers that operate off events and translate environment data to components. Triggers are the way that Wick handles standard use cases and translates them into the component world.
pub(crate) enum TriggerDefinition {
  /// A variant representing a [CliTrigger] type.
  #[serde(rename = "wick/trigger/cli@v1")]
  CliTrigger(CliTrigger),
  /// A variant representing a [HttpTrigger] type.
  #[serde(rename = "wick/trigger/http@v1")]
  HttpTrigger(HttpTrigger),
  /// A variant representing a [TimeTrigger] type.
  #[serde(rename = "wick/trigger/time@v1")]
  TimeTrigger(TimeTrigger),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A trigger that runs when an application is called via the command line.
pub(crate) struct CliTrigger {
  /// The operation that will act as the main entrypoint for this trigger.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A trigger that runs on a schedule similar to cron.
pub(crate) struct TimeTrigger {
  /// The schedule to run the trigger with.
  pub(crate) schedule: Schedule,
  /// The operation to execute on the schedule.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// Values passed to the operation as inputs

  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) payload: Vec<OperationInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Input to use when calling an operation
pub(crate) struct OperationInput {
  /// The name of the input.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The value to pass.

  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) value: Value,
}

#[allow(non_snake_case)]
pub(crate) fn SCHEDULE_REPEAT() -> u16 {
  0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// The schedule to run the Time trigger with.
pub(crate) struct Schedule {
  /// Schedule in cron format with second precision. See [cron.help](https://cron.help) for more information.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) cron: String,
  /// repeat &#x60;n&#x60; times. Use &#x60;0&#x60; to repeat indefinitely

  #[serde(default = "SCHEDULE_REPEAT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) repeat: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to an operation. This type can be shortened to <code>component_id::operation_name</code> with the short-form syntax.
pub(crate) struct ComponentOperationExpression {
  /// The component that exports the operation.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub(crate) component: ComponentDefinition,
  /// The operation name.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Configuration to pass to this operation on invocation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
  /// Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An HTTP server that delegates to HTTP routers on every request.
pub(crate) struct HttpTrigger {
  /// The TcpPort resource to listen on for connections.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) resource: String,
  /// The router to handle incoming requests

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) routers: Vec<HttpRouter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The types of routers that can be configured on the HttpTrigger.
pub(crate) enum HttpRouter {
  /// A variant representing a [RawRouter] type.
  #[serde(rename = "wick/router/raw@v1")]
  RawRouter(RawRouter),
  /// A variant representing a [RestRouter] type.
  #[serde(rename = "wick/router/rest@v1")]
  RestRouter(RestRouter),
  /// A variant representing a [StaticRouter] type.
  #[serde(rename = "wick/router/static@v1")]
  StaticRouter(StaticRouter),
  /// A variant representing a [ProxyRouter] type.
  #[serde(rename = "wick/router/proxy@v1")]
  ProxyRouter(ProxyRouter),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that proxies to the configured URL when the path matches.
pub(crate) struct ProxyRouter {
  /// The path that this router will trigger for.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<Middleware>,
  /// The URL resource to proxy to.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) url: String,
  /// Whether or not to strip the router&#x27;s path from the proxied request.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) strip_path: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that can be configured to delegate to specific operations on a per-route, per-method basis.
pub(crate) struct RestRouter {
  /// The path that this router will trigger for.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// Additional tools and services to enable.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) tools: Option<Tools>,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<Middleware>,
  /// The routes to serve and operations that handle them.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) routes: Vec<Route>,
  /// Information about the router to use when generating documentation and other tools.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) info: Option<Info>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A route to serve and the operation that handles it.
pub(crate) struct Route {
  /// The path to serve this route from. See [URI documentation](/docs/configuration/uri) for more information on specifying query and path parameters.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) uri: String,
  /// The operation that will act as the main entrypoint for this route.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// The HTTP methods to serve this route for.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) methods: Vec<String>,
  /// The name of the route, used for documentation and tooling.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// A short description of the route.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// A longer description of the route.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Additional tools and services to enable.
pub(crate) struct Tools {
  /// The path to serve the OpenAPI spec from

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) openapi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Information about the router to use when generating documentation and other tools.
pub(crate) struct Info {
  /// The title of the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) title: Option<String>,
  /// A short description of the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// The terms of service for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) tos: Option<String>,
  /// The contact information for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) contact: Option<Contact>,
  /// The license information for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) license: Option<License>,
  /// The version of the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) version: Option<String>,
  /// The URL to the API&#x27;s terms of service.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) documentation: Option<Documentation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Information about where and how the API is documented.
pub(crate) struct Documentation {
  /// The URL to the API&#x27;s documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
  /// A short description of the documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Any licensing information for the API.
pub(crate) struct License {
  /// The name of the license.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// The URL to the license.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Contact information to expose for the API.
pub(crate) struct Contact {
  /// The name of the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// The URL to the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) url: Option<String>,
  /// The email address of the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that serves static files.
pub(crate) struct StaticRouter {
  /// The path that this router will trigger for.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<Middleware>,
  /// The volume to serve static files from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) volume: String,
  /// Fallback path (relative to volume &#x60;resource&#x60;) for files to serve in case of a 404. Useful for SPA&#x27;s. if volume resource is: /www and fallback: index.html, then a 404 will serve /www/index.html

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) fallback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that delegates all requests to the configured operation, optionally encoding/decoding based on the specified codec.
pub(crate) struct RawRouter {
  /// The path that this router will trigger for.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) middleware: Option<Middleware>,
  /// The codec to use when encoding/decoding data.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<Codec>,
  /// The operation that handles HTTP requests.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Request and response operations that run before and after the main operation.
pub(crate) struct Middleware {
  /// The middleware to apply to requests.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_component_operation")]
  pub(crate) request: Vec<ComponentOperationExpression>,
  /// The middleware to apply to responses.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_component_operation")]
  pub(crate) response: Vec<ComponentOperationExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A type definition for a Wick Components and Operations
pub(crate) struct TypesConfiguration {
  /// The name of this type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Associated metadata for this type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<Metadata>,
  /// Additional types to export and make available to the type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,
  /// A list of operation signatures.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
  /// Details about the package for this types.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct TestConfiguration {
  /// The name of this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Configuration used to instantiate this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
  /// Unit tests to run against components and operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) cases: Vec<TestDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct ComponentConfiguration {
  /// The name of the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Associated metadata for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<Metadata>,
  /// Details about the package for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageDefinition>,
  /// Configuration for when wick hosts this component as a service.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) host: Option<HostConfig>,
  /// Resources that the component can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,
  /// Components or types to import into this component&#x27;s scope.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ImportBinding>,
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,
  /// Interfaces the component requires to operate.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) requires: Vec<InterfaceBinding>,
  /// Configuration specific to different kinds of components.
  pub(crate) component: ComponentKind,
  /// Assertions that can be run against the component to validate its behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) tests: Vec<TestConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An interface bound to an ID. Used in the require/provide relationship between components.
pub(crate) struct InterfaceBinding {
  /// The name of the interface.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The interface to bind to.
  pub(crate) interface: InterfaceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A interface definition. Used as a signature that components can require or provide.
pub(crate) struct InterfaceDefinition {
  /// Types used by the interface&#x27;s operations

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,
  /// A list of operations defined by this interface.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made from connectiong other components.
pub(crate) struct CompositeComponentConfiguration {
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// A list of operations exposed by the Composite component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<CompositeOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component whose implementation is a WasmRS WebAssembly module.
pub(crate) struct WasmComponentConfiguration {
  /// The path or OCI reference to the WebAssembly module

  #[serde(rename = "ref")]
  pub(crate) reference: crate::v1::helpers::LocationReference,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// A list of operations implemented by the WebAssembly module.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Root component types. These are the components that can be instantiated and run.
pub(crate) enum ComponentKind {
  /// A variant representing a [WasmComponentConfiguration] type.
  #[serde(rename = "wick/component/wasmrs@v1")]
  WasmComponentConfiguration(WasmComponentConfiguration),
  /// A variant representing a [CompositeComponentConfiguration] type.
  #[serde(rename = "wick/component/composite@v1")]
  CompositeComponentConfiguration(CompositeComponentConfiguration),
  /// A variant representing a [SqlComponent] type.
  #[serde(rename = "wick/component/sql@v1")]
  SqlComponent(SqlComponent),
  /// A variant representing a [HttpClientComponent] type.
  #[serde(rename = "wick/component/http@v1")]
  HttpClientComponent(HttpClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Types of possible imports.
pub(crate) enum ImportDefinition {
  /// A variant representing a [TypesComponent] type.
  #[serde(rename = "wick/component/types@v1")]
  TypesComponent(TypesComponent),
  /// A variant representing a [ManifestComponent] type.
  #[serde(rename = "wick/component/manifest@v1")]
  ManifestComponent(ManifestComponent),
  /// A variant representing a [SqlComponent] type.
  #[serde(rename = "wick/component/sql@v1")]
  SqlComponent(SqlComponent),
  /// A variant representing a [HttpClientComponent] type.
  #[serde(rename = "wick/component/http@v1")]
  HttpClientComponent(HttpClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Component types used when referencing operations or linking components.
pub(crate) enum ComponentDefinition {
  /// A variant representing a [GrpcUrlComponent] type.
  #[serde(rename = "wick/component/grpc@v1")]
  GrpcUrlComponent(GrpcUrlComponent),
  /// A variant representing a [ManifestComponent] type.
  #[serde(rename = "wick/component/manifest@v1")]
  ManifestComponent(ManifestComponent),
  /// A variant representing a [ComponentReference] type.
  #[serde(rename = "wick/component/reference@v1")]
  ComponentReference(ComponentReference),
  /// A variant representing a [SqlComponent] type.
  #[serde(rename = "wick/component/sql@v1")]
  SqlComponent(SqlComponent),
  /// A variant representing a [HttpClientComponent] type.
  #[serde(rename = "wick/component/http@v1")]
  HttpClientComponent(HttpClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A types configuration to import into this component's scope.
pub(crate) struct TypesComponent {
  /// The URL (and optional tag) or local file path to find the types manifest.

  #[serde(rename = "ref")]
  pub(crate) reference: crate::v1::helpers::LocationReference,
  /// The types to import from the manifest.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to a component in the application's scope.
pub(crate) struct ComponentReference {
  /// The id of the referenced component.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) id: String,
}

#[allow(non_snake_case)]
pub(crate) fn HOST_CONFIG_TIMEOUT() -> u64 {
  5000
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Host configuration options.
pub(crate) struct HostConfig {
  /// Whether or not to allow the &#x60;:latest&#x60; tag on remote artifacts.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) allow_latest: bool,
  /// A list of registries to connect to insecurely (over HTTP vs HTTPS).

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) insecure_registries: Vec<String>,
  /// The timeout for network requests (in ms).

  #[serde(default = "HOST_CONFIG_TIMEOUT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) timeout: u64,
  /// Configuration for the GRPC server.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) rpc: Option<HttpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for the GRPC service.
pub(crate) struct HttpConfig {
  /// Enable/disable the server.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) enabled: bool,
  /// The port to bind to.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) port: Option<u16>,
  /// The address to bind to.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) address: Option<String>,
  /// Path to pem file for TLS.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) pem: Option<crate::v1::helpers::LocationReference>,
  /// Path to key file for TLS.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) key: Option<crate::v1::helpers::LocationReference>,
  /// Path to CA file.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) ca: Option<crate::v1::helpers::LocationReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component hosted as an independent microservice.
pub(crate) struct GrpcUrlComponent {
  /// The GRPC URL to connect to.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) url: String,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A native component that can be extracted and run as a microservice.
pub(crate) struct ManifestComponent {
  /// The URL (and optional tag) or local file path to find the manifest.

  #[serde(rename = "ref")]
  pub(crate) reference: crate::v1::helpers::LocationReference,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
  /// External components to provide to the referenced component.

  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub(crate) provide: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Composite operations are operations whose implementations come from connecting other operations into a flow or series of pipelines.
pub(crate) struct CompositeOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,
  /// A map of IDs to specific operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) uses: Vec<OperationInstance>,
  /// A list of connections from operation to operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub(crate) flow: Vec<FlowExpression>,
  /// Additional &#x60;CompositeOperationDefinition&#x60;s to define as children.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<CompositeOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
/// A flow operation, i.e. a connection from one operation's outputs to another's inputs.
pub(crate) enum FlowExpression {
  /// A variant representing a [ConnectionDefinition] type.
  ConnectionDefinition(ConnectionDefinition),
  /// A variant representing a [BlockExpression] type.
  BlockExpression(BlockExpression),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A list of FlowExpressions. Typically used only when expanding a shortform `FlowExpression` into multiple `FlowExpression`s.
pub(crate) struct BlockExpression {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub(crate) expressions: Vec<FlowExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(into = "String")]
#[serde(deny_unknown_fields)]
/// A connection between Operations and their ports. This can be specified in short-form syntax.
pub(crate) struct ConnectionDefinition {
  /// An upstream operation&#x27;s output.

  #[serde(deserialize_with = "crate::v1::parse::connection_target_shortform")]
  pub(crate) from: ConnectionTargetDefinition,
  /// A downstream operation&#x27;s input.

  #[serde(deserialize_with = "crate::v1::parse::connection_target_shortform")]
  pub(crate) to: ConnectionTargetDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection target e.g. a specific input or output on an operation instance. This can be specified in shortform syntax.
pub(crate) struct ConnectionTargetDefinition {
  /// The instance ID of the component operation.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) instance: String,
  /// The operation&#x27;s input or output (depending on to/from).

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) port: String,
  /// The default value to provide on this connection in the event of an error.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) data: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An operation name and its input and output signatures
pub(crate) struct OperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Field definition with a name and type signature
pub(crate) struct Field {
  /// The name of the field.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The type signature of the field.

  #[serde(rename = "type")]
  pub(crate) ty: TypeSignature,
  /// The description of the field.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, serde_with::DeserializeFromStr, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(into = "String")]
#[serde(tag = "kind")]
pub(crate) enum TypeSignature {
  /// A variant representing a [I8] type.
  I8(I8),
  /// A variant representing a [I16] type.
  I16(I16),
  /// A variant representing a [I32] type.
  I32(I32),
  /// A variant representing a [I64] type.
  I64(I64),
  /// A variant representing a [U8] type.
  U8(U8),
  /// A variant representing a [U16] type.
  U16(U16),
  /// A variant representing a [U32] type.
  U32(U32),
  /// A variant representing a [U64] type.
  U64(U64),
  /// A variant representing a [F32] type.
  F32(F32),
  /// A variant representing a [F64] type.
  F64(F64),
  /// A variant representing a [Bool] type.
  Bool(Bool),
  /// A variant representing a [StringType] type.
  StringType(StringType),
  /// A variant representing a [Optional] type.
  Optional(Optional),
  /// A variant representing a [Datetime] type.
  Datetime(Datetime),
  /// A variant representing a [Bytes] type.
  Bytes(Bytes),
  /// A variant representing a [Custom] type.
  Custom(Custom),
  /// A variant representing a [List] type.
  List(List),
  /// A variant representing a [Map] type.
  Map(Map),
  /// A variant representing a [Object] type.
  Object(Object),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct F32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct F64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StringType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Datetime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bytes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Custom {
  /// The name of the custom type.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Optional {
  #[serde(rename = "type")]
  pub(crate) ty: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct List {
  #[serde(rename = "type")]
  pub(crate) ty: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Map {
  pub(crate) key: Box<TypeSignature>,

  pub(crate) value: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Object;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// A Struct or Enum type definition.
pub(crate) enum TypeDefinition {
  /// A variant representing a [StructSignature] type.
  #[serde(rename = "wick/type/struct@v1")]
  StructSignature(StructSignature),
  /// A variant representing a [EnumSignature] type.
  #[serde(rename = "wick/type/enum@v1")]
  EnumSignature(EnumSignature),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A struct definition of named fields and types.
pub(crate) struct StructSignature {
  /// The name of the struct.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The fields in this struct.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) fields: Vec<Field>,
  /// The description of the struct.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An enum definition of named variants.
pub(crate) struct EnumSignature {
  /// The name of the enum.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The variants in the enum.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) variants: Vec<EnumVariant>,
  /// The description of the enum.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An enum variant.
pub(crate) struct EnumVariant {
  /// The name of the variant.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The index of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) index: Option<u32>,
  /// The optional value of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) value: Option<String>,
  /// A description of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a component's operation.
pub(crate) struct OperationInstance {
  /// The name of the binding.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The operation to bind to.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// Data to associate with the reference, if any.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
  /// Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A test case for a component's operation.
pub(crate) struct TestDefinition {
  /// The name of the test.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The operaton to test.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) operation: String,
  /// Inherent data to use for the test.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) inherent: Option<InherentData>,
  /// The configuration for the operation, if any.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(deserialize_with = "crate::helpers::configmap_deserializer")]
  pub(crate) with: Option<HashMap<String, Value>>,
  /// The inputs to the test.

  #[serde(default)]
  #[serde(alias = "input")]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<PacketData>,
  /// The expected outputs of the operation.

  #[serde(default)]
  #[serde(alias = "output")]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<PacketData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Data inherent to all invocations.
pub(crate) struct InherentData {
  /// A random seed, i.e. to initialize a random number generator.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) seed: Option<u64>,
  /// A timestamp.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
/// Either a success packet or an error packet.
pub(crate) enum PacketData {
  /// A variant representing a [SuccessPacket] type.
  SuccessPacket(SuccessPacket),
  /// A variant representing a [ErrorPacket] type.
  ErrorPacket(ErrorPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A simplified representation of a Wick data packet & payload, used when writing tests.
pub(crate) struct SuccessPacket {
  /// The name of the input or output this packet is going to or coming from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any flags set on the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) flags: Option<PacketFlags>,
  /// The data to send.

  #[serde(default)]
  #[serde(alias = "data")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ErrorPacket {
  /// The name of the input or output this packet is going to or coming from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any flags set on the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) flags: Option<PacketFlags>,
  /// The error message.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Flags set on a packet.
pub(crate) struct PacketFlags {
  /// Indicates the port should be considered closed.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) done: bool,
  /// Indicates the opening of a new substream context within the parent stream.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) open: bool,
  /// Indicates the closing of a substream context within the parent stream.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) close: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic component whose operations are SQL queries to a database.
pub(crate) struct SqlComponent {
  /// The connect string URL resource for the database.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) resource: String,
  /// Whether or not to use TLS.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) tls: bool,
  /// A list of operations to expose on this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<SqlOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is a SQL query.
pub(crate) struct SqlOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<Field>,
  /// The query to execute.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) query: String,
  /// The positional arguments to the query, defined as a list of input names.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) arguments: Vec<String>,
  /// What to do when an error occurs.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) on_error: Option<ErrorBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// What to do when an error occurs.
pub(crate) enum ErrorBehavior {
  /// The operation will commit what has succeeded.
  Commit = 0,
  /// The operation will rollback changes.
  Rollback = 1,
  /// Errors will be ignored.
  Ignore = 2,
}

impl Default for ErrorBehavior {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for ErrorBehavior {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Commit,
      1 => Self::Rollback,
      2 => Self::Ignore,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Commit,
      1 => Self::Rollback,
      2 => Self::Ignore,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component whose operations are HTTP requests.
pub(crate) struct HttpClientComponent {
  /// The URL base to use.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) resource: String,
  /// The codec to use when encoding/decoding data. Can be overridden by individual operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<Codec>,
  /// A list of operations to expose on this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<HttpClientOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is an HTTP request. The outputs of HttpClientOperationDefinition are always `response` & `body`
pub(crate) struct HttpClientOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<Field>,
  /// The HTTP method to use.

  #[serde(default)]
  pub(crate) method: HttpMethod,
  /// The codec to use when encoding/decoding data.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<Codec>,
  /// Any headers to add to the request.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) headers: Option<HashMap<String, Vec<String>>>,
  /// The body to send, processed as a structured JSON liquid template.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) body: Option<liquid_json::LiquidJsonValue>,
  /// The path to append to our base URL, processed as a liquid template with each input as part of the template data.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Codec to use when encoding/decoding data.
pub(crate) enum Codec {
  /// JSON data
  Json = 0,
  /// Raw
  Raw = 1,
  /// Form Data
  FormData = 2,
}

impl Default for Codec {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for Codec {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Json,
      1 => Self::Raw,
      2 => Self::FormData,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Json,
      1 => Self::Raw,
      2 => Self::FormData,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Supported HTTP methods
pub(crate) enum HttpMethod {
  /// GET method
  Get = 0,
  /// POST method
  Post = 1,
  /// PUT method
  Put = 2,
  /// DELETE method
  Delete = 3,
}

impl Default for HttpMethod {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for HttpMethod {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Get,
      1 => Self::Post,
      2 => Self::Put,
      3 => Self::Delete,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Get,
      1 => Self::Post,
      2 => Self::Put,
      3 => Self::Delete,
      _ => {
        return None;
      }
    })
  }
}
