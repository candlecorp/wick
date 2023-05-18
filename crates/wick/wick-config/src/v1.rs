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
/// Configuration for Wick applications and components.
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
/// The Application configuration defines a standalone Wick application.
pub(crate) struct AppConfiguration {
  /// Associated metadata for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<Metadata>,
  /// The application&#x27;s name.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Configuration that controls how this application runs within a host.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) host: Option<HostConfig>,
  /// Components to import into the application&#x27;s scope.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ImportBinding>,
  /// Resources that the application can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,
  /// Configured triggers that drive the application&#x27;s behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) triggers: Vec<TriggerDefinition>,
  /// Details about the package for this application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// The package details for an application or component.
pub(crate) struct PackageDefinition {
  /// The list of files and folders to be included with the package.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) files: Vec<crate::v1::helpers::Glob>,
  /// Configuration for publishing the package to a registry. This will be used if the package is published without any additional arguments on the command line. If a tag is specified on the command line, that tag will be used instead.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) registry: Option<RegistryDefinition>,
}

#[allow(non_snake_case)]
pub(crate) fn REGISTRY_DEFINITION_REGISTRY() -> String {
  "registry.candle.dev".to_owned()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryDefinition {
  /// The registry to publish to.

  #[serde(default = "REGISTRY_DEFINITION_REGISTRY")]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) registry: String,
  /// The namespace on the registry. ex: registry.candle.dev/&lt;namespace&gt;/&lt;myWickApp&gt;

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Metadata for the component or application.
pub(crate) struct Metadata {
  /// The version of the component or application.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) version: String,
  /// The authors of the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) authors: Vec<String>,
  /// Any vendors associated with the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) vendors: Vec<String>,
  /// A short description of the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// Where to find documentation for the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) documentation: Option<String>,
  /// The license(s) for the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) licenses: Vec<String>,
  /// The icon for the component or application.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) icon: Option<crate::v1::helpers::LocationReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The possible types of resources.
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
/// A trigger called with a CLI context.
pub(crate) struct CliTrigger {
  /// The operation that will act as the main entrypoint for this trigger.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A trigger called with a Time context.
pub(crate) struct TimeTrigger {
  pub(crate) schedule: Schedule,
  /// The operation that will act as the main entrypoint for this trigger.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// Values passed to the operation as inputs

  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) payload: Vec<OperationInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OperationInput {
  /// The name of the operation parameter.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The value to pass to the operation parameter.

  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) value: Value,
}

#[allow(non_snake_case)]
pub(crate) fn SCHEDULE_REPEAT() -> u16 {
  0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Schedule {
  /// schedule in cron format with second precision

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) cron: String,
  /// repeat n times, 0 means forever

  #[serde(default = "SCHEDULE_REPEAT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) repeat: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to an operation with an explicit component definition.
pub(crate) struct ComponentOperationExpression {
  /// The component that exports the operation.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub(crate) component: ComponentDefinition,
  /// The operation to call.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An HTTP server that delegates to HTTP routers upon requests.
pub(crate) struct HttpTrigger {
  /// The TcpPort reference to listen on for connections.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) resource: String,
  /// The HttpRouters that should handle incoming requests

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) routers: Vec<HttpRouter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
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
pub(crate) struct ProxyRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
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
pub(crate) struct RestRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// Additional tools and services to enable.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) tools: Option<Tools>,
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
/// Documentation information for the API.
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
/// The license information for the API.
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
/// The contact information for the API.
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
/// A route to serve and the operation that handles it.
pub(crate) struct Route {
  /// The name of the route, used for documentation and tooling.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// The HTTP methods to serve this route for.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) methods: Vec<String>,
  /// The path to serve this route from.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) uri: String,
  /// The operation that will act as the main entrypoint for this route.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
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
pub(crate) struct StaticRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
  /// The volume to serve static files from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) volume: String,
  /// Fallback path (relative to &#x60;path&#x60;) for files to serve in case of a 404. Useful for SPA&#x27;s. if path: /www and fallback: index.html, then a 404 will serve /www/index.html

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) fallback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A RawHttpRouter delegates raw requests and bodies to operations based on the request path.
pub(crate) struct RawRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
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
#[serde(tag = "kind")]
/// The possible types of resources.
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
/// A filesystem or network volume resource.
pub(crate) struct Volume {
  /// The path.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) path: String,
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
/// A URL configured as a resource.
pub(crate) struct Url {
  /// The url string.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A type definition for a Wick Components and Operations
pub(crate) struct TypesConfiguration {
  /// The name of this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<TypeDefinition>,
  /// A list of operation signatures.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct TestConfiguration {
  /// The name of this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Unit tests to run against components and operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) tests: Vec<TestDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct ComponentConfiguration {
  /// The name of this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// Associated metadata for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) metadata: Option<Metadata>,
  /// Configuration for when wick hosts this component as a service.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) host: Option<HostConfig>,
  /// Resources that the application can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,
  /// Components or types to import into the application&#x27;s scope.

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
  pub(crate) requires: Vec<BoundInterface>,
  /// The labels and values that apply to this manifest.

  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub(crate) labels: HashMap<String, String>,
  /// Configuration specific to different kinds of components.
  pub(crate) component: ComponentKind,
  /// Assertions that can be run against the component to validate its behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) tests: Vec<TestDefinition>,
  /// Details about the package for this component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) package: Option<PackageDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An interface bound to an ID.
pub(crate) struct BoundInterface {
  /// The name of the interface.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// The interface to bind to.
  pub(crate) interface: InterfaceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A generic interface definition.
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
/// A component made out of other components
pub(crate) struct CompositeComponentConfiguration {
  /// A list of operations implemented by the Composite component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<CompositeOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made out of other components
pub(crate) struct WasmComponentConfiguration {
  /// A reference to a local WebAssembly implementation

  #[serde(rename = "ref")]
  pub(crate) reference: crate::v1::helpers::LocationReference,
  /// A list of operations implemented by the WebAssembly module.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
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
/// Component implementation types
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
/// A types manifest to import into this component's scope.
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
  /// The id of the component to reference.

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
  /// Whether or not to allow the :latest tag on remote artifacts.

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
/// A definition for a single composite operation.
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
  /// A list of components the schematic can use.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) components: Vec<String>,
  /// A map of IDs to specific operations.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) uses: Vec<InstanceBinding>,
  /// A list of connections from operation to operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub(crate) flow: Vec<FlowExpression>,
  /// Additional flow operations scoped to this operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<CompositeOperationDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// A flow operation, i.e. a connection from one operation to another.
pub(crate) enum FlowExpression {
  /// A variant representing a [ConnectionDefinition] type.
  ConnectionDefinition(ConnectionDefinition),
  /// A variant representing a [BlockExpression] type.
  BlockExpression(BlockExpression),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A list of FlowExpressions
pub(crate) struct BlockExpression {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub(crate) expressions: Vec<FlowExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection between Operations and their ports. This can be specified in short-form syntax (where applicable).
pub(crate) struct ConnectionDefinition {
  /// The upstream operation port.

  #[serde(deserialize_with = "crate::v1::parse::connection_target_shortform")]
  pub(crate) from: ConnectionTargetDefinition,
  /// The downstream operation port.

  #[serde(deserialize_with = "crate::v1::parse::connection_target_shortform")]
  pub(crate) to: ConnectionTargetDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable).
pub(crate) struct ConnectionTargetDefinition {
  /// The instance ID of the operation.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) instance: String,
  /// The operation port.

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
pub(crate) struct I8();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I16();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I32();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct I64();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U8();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U16();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U32();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct U64();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct F32();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct F64();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bool();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StringType();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Datetime();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bytes();

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
pub(crate) struct Object();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// Struct or Enum type definition.
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
  /// The description of the variant.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a component's operation.
pub(crate) struct InstanceBinding {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A test case for a component.
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
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) input: Vec<PacketData>,
  /// The expected outputs of the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) output: Vec<PacketData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Data inherent to transactions.
pub(crate) struct InherentData {
  /// An RNG seed.

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
  /// A variant representing a [PayloadData] type.
  PayloadData(PayloadData),
  /// A variant representing a [ErrorData] type.
  ErrorData(ErrorData),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A simplified representation of a Wick data packet & payload, used to write tests.
pub(crate) struct PayloadData {
  /// The name of the port to send the data to.

  #[serde(deserialize_with = "crate::helpers::with_expand_envs_string")]
  pub(crate) name: String,
  /// Any flags set on the packet.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) flags: Option<PacketFlags>,
  /// The data to send.

  #[serde(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ErrorData {
  /// The name of the port to send the data to.

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
  /// When set, indicates the port should be considered closed.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) done: bool,
  /// When set, indicates the opening of a new substream context within the parent stream.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) open: bool,
  /// When set, indicates the closing of a substream context within the parent stream.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) close: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made out of other components
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
  /// The arguments to the query, defined as a list of input names.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made out of other components
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
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) headers: HashMap<String, Vec<String>>,
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
  /// JSON Codec
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
  Get = 0,
  Post = 1,
  Put = 2,
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
