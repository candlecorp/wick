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
  clippy::explicit_deref_methods
)]
#![warn(clippy::cognitive_complexity)]
#![allow(
  missing_docs,
  clippy::large_enum_variant,
  missing_copy_implementations,
  clippy::enum_variant_names
)]

#[cfg(feature = "config")]
pub(crate) mod conversions;
pub mod helpers;
pub(crate) mod parse;

use std::collections::HashMap;

use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Root configuration that can be any one of the possible Wick configuration formats.
pub enum WickConfig {
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
  /// A variant representing a [LockdownConfiguration] type.
  #[serde(rename = "wick/lockdown@v1")]
  LockdownConfiguration(LockdownConfiguration),
}

/// A liquid template. Liquid-JSON is a way of using Liquid templates in structured JSON-like data. See liquid's [homepage](https://shopify.github.io/liquid/) for more information.
#[allow(unused)]
pub type LiquidTemplate = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for a standalone Wick application.
pub struct AppConfiguration {
  /// The application&#x27;s name.

  #[serde(default)]
  pub name: String,
  /// Associated metadata for this application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,
  /// Details about the package for this application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub package: Option<PackageDefinition>,
  /// Resources and configuration that the application and its components can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<ResourceBinding>,
  /// Components that to import and make available to the application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub import: Vec<ImportBinding>,
  /// Triggers to load and instantiate to drive the application&#x27;s behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub triggers: Vec<TriggerDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Metadata to associate with an artifact.
pub struct Metadata {
  /// The version of the artifact.

  #[serde(default)]
  pub version: String,
  /// A list of the authors.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub authors: Vec<String>,
  /// A list of any vendors associated with the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub vendors: Vec<String>,
  /// A short description.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Where to find documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub documentation: Option<String>,
  /// The license(s) for the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub licenses: Vec<String>,
  /// An icon to associate with the artifact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub icon: Option<crate::v1::helpers::LocationReference>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for packaging and publishing Wick configurations.
pub struct PackageDefinition {
  /// The list of files and folders to be included with the package.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub files: Vec<crate::v1::helpers::Glob>,
  /// Configuration for publishing the package to a registry.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registry: Option<RegistryDefinition>,
}

#[allow(non_snake_case)]
pub(crate) fn REGISTRY_DEFINITION_HOST() -> String {
  "registry.candle.dev".to_owned()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistryDefinition {
  /// The registry to publish to, e.g. registry.candle.dev

  #[serde(default = "REGISTRY_DEFINITION_HOST")]
  #[serde(alias = "registry")]
  pub host: String,
  /// The namespace on the registry. e.g.: [*your username*]

  #[serde(default)]
  pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a resource.
pub struct ResourceBinding {
  /// The name of the binding.
  pub name: String,
  /// The resource to bind to.
  pub resource: ResourceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to an imported component or type manifest.
pub struct ImportBinding {
  /// The name of the binding.
  pub name: String,
  /// The import to bind to.
  pub component: ImportDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The possible types of resources. Resources are system-level resources and sensitive configuration.
pub enum ResourceDefinition {
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
pub struct TcpPort {
  /// The port to bind to.

  #[serde(default)]
  pub port: LiquidTemplate,
  /// The address to bind to.

  #[serde(default)]
  pub address: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A UDP port to bind to.
pub struct UdpPort {
  /// The port to bind to.

  #[serde(default)]
  pub port: LiquidTemplate,
  /// The address to bind to.

  #[serde(default)]
  pub address: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A filesystem or network volume resource.
pub struct Volume {
  /// The path.
  pub path: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A URL configured as a resource.
pub struct Url {
  /// The url string.
  pub url: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Triggers that operate off events and translate environment data to components. Triggers are the way that Wick handles standard use cases and translates them into the component world.
pub enum TriggerDefinition {
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
pub struct CliTrigger {
  /// The operation that will act as the main entrypoint for this trigger.

  #[serde(serialize_with = "crate::v1::helpers::serialize_component_expression")]
  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub operation: ComponentOperationExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A trigger that runs on a schedule similar to cron.
pub struct TimeTrigger {
  /// The schedule to run the trigger with.
  pub schedule: Schedule,
  /// The operation to execute on the schedule.

  #[serde(serialize_with = "crate::v1::helpers::serialize_component_expression")]
  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub operation: ComponentOperationExpression,
  /// Values passed to the operation as inputs

  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub payload: Vec<OperationInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Input to use when calling an operation
pub struct OperationInput {
  /// The name of the input.
  pub name: String,
  /// The value to pass.
  pub value: Value,
}

#[allow(non_snake_case)]
pub(crate) fn SCHEDULE_REPEAT() -> u16 {
  0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// The schedule to run the Time trigger with.
pub struct Schedule {
  /// Schedule in cron format with second precision. See [cron.help](https://cron.help) for more information.
  pub cron: String,
  /// repeat &#x60;n&#x60; times. Use &#x60;0&#x60; to repeat indefinitely

  #[serde(default = "SCHEDULE_REPEAT")]
  pub repeat: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to an operation. This type can be shortened to <code>component_id::operation_name</code> with the short-form syntax.
pub struct ComponentOperationExpression {
  /// The component that exports the operation.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub component: ComponentDefinition,
  /// The operation name.
  pub name: String,
  /// Configuration to pass to this operation on invocation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
  /// Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An HTTP server that delegates to HTTP routers on every request.
pub struct HttpTrigger {
  /// The TcpPort resource to listen on for connections.
  pub resource: String,
  /// The router to handle incoming requests

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub routers: Vec<HttpRouter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The types of routers that can be configured on the HttpTrigger.
pub enum HttpRouter {
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
pub struct ProxyRouter {
  /// The path that this router will trigger for.
  pub path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub middleware: Option<Middleware>,
  /// The URL resource to proxy to.
  pub url: String,
  /// Whether or not to strip the router&#x27;s path from the proxied request.

  #[serde(default)]
  pub strip_path: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that can be configured to delegate to specific operations on a per-route, per-method basis.
pub struct RestRouter {
  /// The path that this router will trigger for.
  pub path: String,
  /// Additional tools and services to enable.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools: Option<Tools>,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub middleware: Option<Middleware>,
  /// The routes to serve and operations that handle them.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub routes: Vec<Route>,
  /// Information about the router to use when generating documentation and other tools.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub info: Option<Info>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A route to serve and the operation that handles it.
pub struct Route {
  /// The path to serve this route from. See [URI documentation](/docs/configuration/uri) for more information on specifying query and path parameters.

  #[serde(alias = "uri")]
  pub sub_path: String,
  /// The operation that will act as the main entrypoint for this route.

  #[serde(serialize_with = "crate::v1::helpers::serialize_component_expression")]
  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub operation: ComponentOperationExpression,
  /// The HTTP methods to serve this route for.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub methods: Vec<HttpMethod>,
  /// The unique ID of the route, used for documentation and tooling.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  /// A short description of the route.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// A longer description of the route.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Additional tools and services to enable.
pub struct Tools {
  /// Set to true to generate an OpenAPI specification and serve it at *router_path*/openapi.json

  #[serde(default)]
  pub openapi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Information about the router to use when generating documentation and other tools.
pub struct Info {
  /// The title of the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
  /// A short description of the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// The terms of service for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tos: Option<String>,
  /// The contact information for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub contact: Option<Contact>,
  /// The license information for the API.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub license: Option<License>,
  /// The version of the API.

  #[serde(default)]
  pub version: String,
  /// The URL to the API&#x27;s terms of service.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub documentation: Option<Documentation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Information about where and how the API is documented.
pub struct Documentation {
  /// The URL to the API&#x27;s documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  /// A short description of the documentation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Any licensing information for the API.
pub struct License {
  /// The name of the license.
  pub name: String,
  /// The URL to the license.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Contact information to expose for the API.
pub struct Contact {
  /// The name of the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// The URL to the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  /// The email address of the contact.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that serves static files.
pub struct StaticRouter {
  /// The path that this router will trigger for.
  pub path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub middleware: Option<Middleware>,
  /// The volume to serve static files from.
  pub volume: String,
  /// Fallback path (relative to volume &#x60;resource&#x60;) for files to serve in case of a 404. Useful for SPA&#x27;s. if volume resource is: /www and fallback: index.html, then a 404 will serve /www/index.html

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fallback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A router that delegates all requests to the configured operation, optionally encoding/decoding based on the specified codec.
pub struct RawRouter {
  /// The path that this router will trigger for.
  pub path: String,
  /// Middleware operations for this router.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub middleware: Option<Middleware>,
  /// The codec to use when encoding/decoding data.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub codec: Option<Codec>,
  /// The operation that handles HTTP requests.

  #[serde(serialize_with = "crate::v1::helpers::serialize_component_expression")]
  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub operation: ComponentOperationExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Request and response operations that run before and after the main operation.
pub struct Middleware {
  /// The middleware to apply to requests.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_component_operation")]
  pub request: Vec<ComponentOperationExpression>,
  /// The middleware to apply to responses.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_component_operation")]
  pub response: Vec<ComponentOperationExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A type definition for a Wick Components and Operations
pub struct TypesConfiguration {
  /// The name of this type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// Associated metadata for this type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,
  /// Additional types to export and make available to the type.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<TypeDefinition>,
  /// A list of operation signatures.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<OperationDefinition>,
  /// Details about the package for this types.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub package: Option<PackageDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub struct TestConfiguration {
  /// The name of this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// Configuration used to instantiate this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
  /// Unit tests to run against components and operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub cases: Vec<TestDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A lockdown configuration used to secure Wick components and applications
pub struct LockdownConfiguration {
  /// Associated metadata for this configuration.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,
  /// Restrictions to apply to resources before an application or component can be run.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<ResourceRestriction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Restrictions to assert against an application or component.
pub enum ResourceRestriction {
  /// A variant representing a [VolumeRestriction] type.
  #[serde(rename = "wick/resource/volume@v1")]
  VolumeRestriction(VolumeRestriction),
  /// A variant representing a [UrlRestriction] type.
  #[serde(rename = "wick/resource/url@v1")]
  UrlRestriction(UrlRestriction),
  /// A variant representing a [TcpPortRestriction] type.
  #[serde(rename = "wick/resource/tcpport@v1")]
  TcpPortRestriction(TcpPortRestriction),
  /// A variant representing a [UdpPortRestriction] type.
  #[serde(rename = "wick/resource/udpport@v1")]
  UdpPortRestriction(UdpPortRestriction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Restrictions to apply against Volume resources
pub struct VolumeRestriction {
  /// The components this restriction applies to

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// The volumes to allow
  pub allow: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Restrictions to apply against URL resources
pub struct UrlRestriction {
  /// The components this restriction applies to

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// The URLs to allow
  pub allow: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Restrictions to apply against TCP Port resources
pub struct TcpPortRestriction {
  /// The components this restriction applies to

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// The address to allow
  pub address: LiquidTemplate,
  /// The port to allow
  pub port: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Restrictions to apply against UDP Port resources
pub struct UdpPortRestriction {
  /// The components this restriction applies to

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// The address to allow
  pub address: LiquidTemplate,
  /// The port to allow
  pub port: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub struct ComponentConfiguration {
  /// The name of the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// Associated metadata for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,
  /// Details about the package for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub package: Option<PackageDefinition>,
  /// Configuration for when wick hosts this component as a service.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub host: Option<HostConfig>,
  /// Resources that the component can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<ResourceBinding>,
  /// Components or types to import into this component&#x27;s scope.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub import: Vec<ImportBinding>,
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<TypeDefinition>,
  /// Interfaces the component requires to operate.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub requires: Vec<InterfaceBinding>,
  /// Configuration specific to different kinds of components.
  pub component: ComponentKind,
  /// Assertions that can be run against the component to validate its behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub tests: Vec<TestConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An interface bound to an ID. Used in the require/provide relationship between components.
pub struct InterfaceBinding {
  /// The name of the interface.
  pub name: String,
  /// The interface to bind to.
  pub interface: InterfaceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A interface definition. Used as a signature that components can require or provide.
pub struct InterfaceDefinition {
  /// Types used by the interface&#x27;s operations

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<TypeDefinition>,
  /// A list of operations defined by this interface.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<OperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made from connectiong other components.
pub struct CompositeComponentConfiguration {
  /// A list of operations exposed by the Composite component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<CompositeOperationDefinition>,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// A component or components whose operations you want to inherit from.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub extends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component whose implementation is a WasmRS WebAssembly module.
pub struct WasmComponentConfiguration {
  /// The path or OCI reference to the WebAssembly module

  #[serde(rename = "ref")]
  pub reference: crate::v1::helpers::LocationReference,
  /// Volumes to expose to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub volumes: Vec<ExposedVolume>,
  /// The default size to allocate to the component&#x27;s send/receive buffer.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_packet_size: Option<u32>,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// A list of operations implemented by the WebAssembly module.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<OperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Volumes to expose to a component and the internal paths they map to.
pub struct ExposedVolume {
  /// The resource ID of the volume.
  pub resource: String,
  /// The path to map it to in the component.
  pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Root component types. These are the components that can be instantiated and run.
pub enum ComponentKind {
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
  /// A variant representing a [WebSocketClientComponent] type.
  #[serde(rename = "wick/component/websocket@v1")]
  WebSocketClientComponent(WebSocketClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Types of possible imports.
pub enum ImportDefinition {
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
  /// A variant representing a [WebSocketClientComponent] type.
  #[serde(rename = "wick/component/websocket@v1")]
  WebSocketClientComponent(WebSocketClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Component types used when referencing operations or linking components.
pub enum ComponentDefinition {
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
  /// A variant representing a [WebSocketClientComponent] type.
  #[serde(rename = "wick/component/websocket@v1")]
  WebSocketClientComponent(WebSocketClientComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A types configuration to import into this component's scope.
pub struct TypesComponent {
  /// The URL (and optional tag) or local file path to find the types manifest.

  #[serde(rename = "ref")]
  pub reference: crate::v1::helpers::LocationReference,
  /// The types to import from the manifest.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to a component in the application's scope.
pub struct ComponentReference {
  /// The id of the referenced component.
  pub id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Host configuration options.
pub struct HostConfig {
  /// Whether or not to allow the &#x60;:latest&#x60; tag on remote artifacts.

  #[serde(default)]
  pub allow_latest: bool,
  /// A list of registries to connect to insecurely (over HTTP vs HTTPS).

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub insecure_registries: Vec<String>,
  /// Configuration for the GRPC server.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rpc: Option<HttpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for the GRPC service.
pub struct HttpConfig {
  /// Enable/disable the server.

  #[serde(default)]
  pub enabled: bool,
  /// The port to bind to.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub port: Option<u16>,
  /// The address to bind to.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address: Option<String>,
  /// Path to pem file for TLS.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pem: Option<crate::v1::helpers::LocationReference>,
  /// Path to key file for TLS.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub key: Option<crate::v1::helpers::LocationReference>,
  /// Path to CA file.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ca: Option<crate::v1::helpers::LocationReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component hosted as an independent microservice.
pub struct GrpcUrlComponent {
  /// The GRPC URL to connect to.
  pub url: String,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration defined in a Wick component manifest.
pub struct ManifestComponent {
  /// The URL (and optional tag) or local file path to find the manifest.

  #[serde(rename = "ref")]
  pub reference: crate::v1::helpers::LocationReference,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
  /// External components to provide to the referenced component.

  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub provide: HashMap<String, String>,
  /// If applicable, the default size to allocate to the component&#x27;s send/receive buffer.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_packet_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Composite operations are operations whose implementations come from connecting other operations into a flow or series of pipelines.
pub struct CompositeOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<Field>,
  /// A map of IDs to specific operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub uses: Vec<OperationInstance>,
  /// A list of connections from operation to operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub flow: Vec<FlowExpression>,
  /// Additional &#x60;CompositeOperationDefinition&#x60;s to define as children.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<CompositeOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
/// A flow operation, i.e. a connection from one operation's outputs to another's inputs.
pub enum FlowExpression {
  /// A variant representing a [ConnectionDefinition] type.
  #[serde(rename = "ConnectionDefinition")]
  ConnectionDefinition(ConnectionDefinition),
  /// A variant representing a [BlockExpression] type.
  #[serde(rename = "BlockExpression")]
  BlockExpression(BlockExpression),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A list of FlowExpressions. Typically used only when expanding a shortform `FlowExpression` into multiple `FlowExpression`s.
pub struct BlockExpression {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub expressions: Vec<FlowExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(into = "String")]
#[serde(deny_unknown_fields)]
/// A connection between Operations and their ports. This can be specified in short-form syntax.
pub struct ConnectionDefinition {
  /// An upstream operation&#x27;s output.
  pub from: ConnectionTargetDefinition,
  /// A downstream operation&#x27;s input.
  pub to: ConnectionTargetDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection target e.g. a specific input or output on an operation instance. This can be specified in shortform syntax.
pub struct ConnectionTargetDefinition {
  /// The instance ID of the component operation.
  pub instance: String,
  /// The operation&#x27;s input or output (depending on to/from).

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub port: Option<String>,
  /// The default value to provide on this connection in the event of an error.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An operation name and its input and output signatures
pub struct OperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Field definition with a name and type signature
pub struct Field {
  /// The name of the field.
  pub name: String,
  /// The type signature of the field.

  #[serde(rename = "type")]
  pub ty: TypeSignature,
  /// The description of the field.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, serde_with::DeserializeFromStr, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(into = "String")]
#[serde(tag = "kind")]
pub enum TypeSignature {
  /// A variant representing a [I8] type.
  #[serde(rename = "I8")]
  I8(I8),
  /// A variant representing a [I16] type.
  #[serde(rename = "I16")]
  I16(I16),
  /// A variant representing a [I32] type.
  #[serde(rename = "I32")]
  I32(I32),
  /// A variant representing a [I64] type.
  #[serde(rename = "I64")]
  I64(I64),
  /// A variant representing a [U8] type.
  #[serde(rename = "U8")]
  U8(U8),
  /// A variant representing a [U16] type.
  #[serde(rename = "U16")]
  U16(U16),
  /// A variant representing a [U32] type.
  #[serde(rename = "U32")]
  U32(U32),
  /// A variant representing a [U64] type.
  #[serde(rename = "U64")]
  U64(U64),
  /// A variant representing a [F32] type.
  #[serde(rename = "F32")]
  F32(F32),
  /// A variant representing a [F64] type.
  #[serde(rename = "F64")]
  F64(F64),
  /// A variant representing a [Bool] type.
  #[serde(rename = "Bool")]
  Bool(Bool),
  /// A variant representing a [StringType] type.
  #[serde(rename = "StringType")]
  StringType(StringType),
  /// A variant representing a [Optional] type.
  #[serde(rename = "Optional")]
  Optional(Optional),
  /// A variant representing a [Datetime] type.
  #[serde(rename = "Datetime")]
  Datetime(Datetime),
  /// A variant representing a [Bytes] type.
  #[serde(rename = "Bytes")]
  Bytes(Bytes),
  /// A variant representing a [Custom] type.
  #[serde(rename = "Custom")]
  Custom(Custom),
  /// A variant representing a [List] type.
  #[serde(rename = "List")]
  List(List),
  /// A variant representing a [Map] type.
  #[serde(rename = "Map")]
  Map(Map),
  /// A variant representing a [Object] type.
  #[serde(rename = "Object")]
  Object(Object),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct I8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct I16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct I32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct I64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct U8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct U16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct U32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct U64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct F32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct F64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StringType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Datetime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bytes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Custom {
  /// The name of the custom type.

  #[serde(default)]
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Optional {
  #[serde(rename = "type")]
  pub ty: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct List {
  #[serde(rename = "type")]
  pub ty: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Map {
  pub key: Box<TypeSignature>,

  pub value: Box<TypeSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Object;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// A Struct or Enum type definition.
pub enum TypeDefinition {
  /// A variant representing a [StructSignature] type.
  #[serde(rename = "wick/type/struct@v1")]
  StructSignature(StructSignature),
  /// A variant representing a [EnumSignature] type.
  #[serde(rename = "wick/type/enum@v1")]
  EnumSignature(EnumSignature),
  /// A variant representing a [UnionSignature] type.
  #[serde(rename = "wick/type/union@v1")]
  UnionSignature(UnionSignature),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A struct definition of named fields and types.
pub struct StructSignature {
  /// The name of the struct.

  #[serde(default)]
  pub name: String,
  /// The fields in this struct.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub fields: Vec<Field>,
  /// The description of the struct.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An enum definition of named variants.
pub struct UnionSignature {
  /// The name of the union.

  #[serde(default)]
  pub name: String,
  /// The types in the union.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub types: Vec<TypeSignature>,
  /// The description of the union.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An enum definition of named variants.
pub struct EnumSignature {
  /// The name of the enum.

  #[serde(default)]
  pub name: String,
  /// The variants in the enum.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub variants: Vec<EnumVariant>,
  /// The description of the enum.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An enum variant.
pub struct EnumVariant {
  /// The name of the variant.

  #[serde(default)]
  pub name: String,
  /// The index of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub index: Option<u32>,
  /// The optional value of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
  /// A description of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a component's operation.
pub struct OperationInstance {
  /// The name of the binding.
  pub name: String,
  /// The operation to bind to.

  #[serde(serialize_with = "crate::v1::helpers::serialize_component_expression")]
  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub operation: ComponentOperationExpression,
  /// Data to associate with the reference, if any.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
  /// Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A test case for a component's operation.
pub struct TestDefinition {
  /// The name of the test.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// The operaton to test.
  pub operation: String,
  /// Inherent data to use for the test.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inherent: Option<InherentData>,
  /// The configuration for the operation, if any.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub with: Option<HashMap<String, liquid_json::LiquidJsonValue>>,
  /// The inputs to the test.

  #[serde(default)]
  #[serde(alias = "input")]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<PacketData>,
  /// The expected outputs of the operation.

  #[serde(default)]
  #[serde(alias = "output")]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<TestPacketData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Data inherent to all invocations.
pub struct InherentData {
  /// A random seed, i.e. to initialize a random number generator.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub seed: Option<u64>,
  /// A timestamp.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
/// Either a success packet or an error packet.
pub enum PacketData {
  /// A variant representing a [SuccessPacket] type.
  #[serde(rename = "SuccessPacket")]
  SuccessPacket(SuccessPacket),
  /// A variant representing a [ErrorPacket] type.
  #[serde(rename = "ErrorPacket")]
  ErrorPacket(ErrorPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
/// Packet assertions.
pub enum TestPacketData {
  /// A variant representing a [SuccessPacket] type.
  #[serde(rename = "SuccessPacket")]
  SuccessPacket(SuccessPacket),
  /// A variant representing a [PacketAssertionDef] type.
  #[serde(rename = "PacketAssertionDef")]
  PacketAssertionDef(PacketAssertionDef),
  /// A variant representing a [ErrorPacket] type.
  #[serde(rename = "ErrorPacket")]
  ErrorPacket(ErrorPacket),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A simplified representation of a Wick data packet & payload, used when writing tests.
pub struct SuccessPacket {
  /// The name of the input or output this packet is going to or coming from.
  pub name: String,
  /// Any flags set on the packet. Deprecated, use &#x27;flag:&#x27; instead

  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flags: Option<PacketFlags>,
  /// The flag set on the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flag: Option<PacketFlag>,
  /// The packet payload.

  #[serde(default)]
  #[serde(alias = "data")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<liquid_json::LiquidJsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A test case for a component's operation that uses loose equality for comparing data.
pub struct PacketAssertionDef {
  /// The name of the input or output this packet is going to or coming from.
  pub name: String,
  /// An assertion to test against the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub assertions: Vec<PacketAssertion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A packet assertion.
pub struct PacketAssertion {
  /// The optional path to a value in the packet to assert against.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// The operation to use when asserting against a packet.
  pub operator: AssertionOperator,
  /// A value or object combine with the operator to assert against a packet value.
  pub value: liquid_json::LiquidJsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// An operation that drives the logic in a packet assertion.
pub enum AssertionOperator {
  Equals = 0,
  LessThan = 1,
  GreaterThan = 2,
  Regex = 3,
  Contains = 4,
}

impl Default for AssertionOperator {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for AssertionOperator {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Equals,
      1 => Self::LessThan,
      2 => Self::GreaterThan,
      3 => Self::Regex,
      4 => Self::Contains,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Equals,
      1 => Self::LessThan,
      2 => Self::GreaterThan,
      3 => Self::Regex,
      4 => Self::Contains,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ErrorPacket {
  /// The name of the input or output this packet is going to or coming from.
  pub name: String,
  /// Any flags set on the packet. Deprecated, use &#x27;flag:&#x27; instead

  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flags: Option<PacketFlags>,
  /// The flag set on the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flag: Option<PacketFlag>,
  /// The error message.
  pub error: LiquidTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Flags set on a packet.
pub struct PacketFlags {
  /// Indicates the port should be considered closed.

  #[serde(default)]
  pub done: bool,
  /// Indicates the opening of a new substream context within the parent stream.

  #[serde(default)]
  pub open: bool,
  /// Indicates the closing of a substream context within the parent stream.

  #[serde(default)]
  pub close: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Possible flags that can be set on a packet.
pub enum PacketFlag {
  /// Indicates the port should be considered closed.
  Done = 0,
  /// Indicates the opening of a new substream context within the parent stream.
  Open = 1,
  /// Indicates the closing of a substream context within the parent stream.
  Close = 2,
}

impl Default for PacketFlag {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for PacketFlag {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Done,
      1 => Self::Open,
      2 => Self::Close,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Done,
      1 => Self::Open,
      2 => Self::Close,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic component whose operations are SQL queries to a database.
pub struct SqlComponent {
  /// The connect string URL resource for the database.

  #[serde(default)]
  pub resource: String,
  /// Whether or not to use TLS.

  #[serde(default)]
  pub tls: bool,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// A list of operations to expose on this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<SqlQueryKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum SqlQueryKind {
  /// A variant representing a [SqlQueryOperationDefinition] type.
  #[serde(rename = "SqlQueryOperationDefinition")]
  SqlQueryOperationDefinition(SqlQueryOperationDefinition),
  /// A variant representing a [SqlExecOperationDefinition] type.
  #[serde(rename = "SqlExecOperationDefinition")]
  SqlExecOperationDefinition(SqlExecOperationDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is a SQL query.
pub struct SqlQueryOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<Field>,
  /// The query to execute.
  pub query: String,
  /// The positional arguments to the query, defined as a list of input names.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub arguments: Vec<String>,
  /// What to do when an error occurs.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub on_error: Option<ErrorBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is a SQL query that returns the number of rows affected or failure.
pub struct SqlExecOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<Field>,
  /// The query to execute.
  pub exec: String,
  /// The positional arguments to the query, defined as a list of input names.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub arguments: Vec<String>,
  /// What to do when an error occurs.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub on_error: Option<ErrorBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// What to do when an error occurs.
pub enum ErrorBehavior {
  /// Errors will be ignored.
  Ignore = 0,
  /// The operation will commit what has succeeded.
  Commit = 1,
  /// The operation will rollback changes.
  Rollback = 2,
}

impl Default for ErrorBehavior {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for ErrorBehavior {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Ignore,
      1 => Self::Commit,
      2 => Self::Rollback,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Ignore,
      1 => Self::Commit,
      2 => Self::Rollback,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component whose operations are HTTP requests.
pub struct HttpClientComponent {
  /// The URL base to use.

  #[serde(default)]
  pub resource: String,
  /// The codec to use when encoding/decoding data. Can be overridden by individual operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub codec: Option<Codec>,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// A list of operations to expose on this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<HttpClientOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is an HTTP request. The outputs of HttpClientOperationDefinition are always `response` & `body`
pub struct HttpClientOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// The HTTP method to use.

  #[serde(default)]
  pub method: HttpMethod,
  /// The codec to use when encoding/decoding data.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub codec: Option<Codec>,
  /// Any headers to add to the request.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub headers: Option<HashMap<String, Vec<String>>>,
  /// The body to send, processed as a structured JSON liquid template.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub body: Option<liquid_json::LiquidJsonValue>,
  /// The path to append to our base URL, processed as a liquid template with each input as part of the template data.

  #[serde(default)]
  pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Codec to use when encoding/decoding data.
pub enum Codec {
  /// JSON data
  Json = 0,
  /// Raw bytes
  Raw = 1,
  /// Form Data
  FormData = 2,
  /// Raw text
  Text = 3,
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
      3 => Self::Text,
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
      3 => Self::Text,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Supported HTTP methods
pub enum HttpMethod {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component whose operations are WebSocket connection.
pub struct WebSocketClientComponent {
  /// The URL of the WebSocket server.

  #[serde(default)]
  pub resource: String,
  /// Configuration necessary to provide when instantiating the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// A list of operations to expose on this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub operations: Vec<WebSocketClientOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A dynamic operation whose implementation is a WebSocket message. The outputs of WebSocketOperationDefinition are always `message`.
pub struct WebSocketClientOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  pub name: String,
  /// Any configuration required by the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub with: Vec<Field>,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub inputs: Vec<Field>,
  /// The path / query string to append to our base URL, processed as a liquid template with each input as part of the template data.

  #[serde(default)]
  pub path: String,
  /// The message to send, processed as a structured JSON liquid template.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<liquid_json::LiquidJsonValue>,
}
