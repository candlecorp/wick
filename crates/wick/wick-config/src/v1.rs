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
pub(crate) mod impls;
pub(crate) mod parse;

use std::collections::HashMap;

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
  /// The configuration version.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) format: u32,
  /// Associated metadata for this component.

  #[serde(default)]
  pub(crate) metadata: Option<AppMetadata>,
  /// The application&#x27;s name.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// Configuration that controls how this application runs within a host.

  #[serde(default)]
  pub(crate) host: HostConfig,
  /// Components to import into the application&#x27;s scope.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ComponentBinding>,
  /// Resources that the application can access.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) resources: Vec<ResourceBinding>,
  /// Configured triggers that drive the application&#x27;s behavior.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) triggers: Vec<TriggerDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Metadata for the application.
pub(crate) struct AppMetadata {
  /// The version of the application.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) version: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A trigger called with a CLI context.
pub(crate) struct CliTrigger {
  /// The operation that will act as the main entrypoint for this trigger.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// The component that provides additional logic.

  #[serde(default)]
  pub(crate) app: Option<ComponentDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to an operation with an explicit component definition.
pub(crate) struct ComponentOperationExpression {
  /// The component that exports the operation.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub(crate) component: ComponentDefinition,
  /// The operation to call.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An HTTP server that delegates to HTTP routers upon requests.
pub(crate) struct HttpTrigger {
  /// The TcpPort reference to listen on for connections.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RestRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) path: String,
  /// The component to expose as a Rest API.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub(crate) component: ComponentDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A RawHttpRouter delegates raw requests and bodies to operations based on the request path.
pub(crate) struct RawRouter {
  /// The path to start serving this router from.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) path: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A WebAssembly component.
pub(crate) struct TcpPort {
  /// The port to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: u16,
  /// The address to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A WebAssembly component.
pub(crate) struct UdpPort {
  /// The port to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: u16,
  /// The address to bind to.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct TypesConfiguration {
  /// The name of this component.

  #[serde(default)]
  pub(crate) name: Option<String>,
  /// The component manifest format version

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) format: u32,
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<wick_interface_types::TypeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A configuration for a Wick Component
pub(crate) struct TestConfiguration {
  /// The name of this component.

  #[serde(default)]
  pub(crate) name: Option<String>,
  /// The component manifest format version

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) format: u32,
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
  pub(crate) name: Option<String>,
  /// The component manifest format version

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) format: u32,
  /// Associated metadata for this component.

  #[serde(default)]
  pub(crate) metadata: Option<ComponentMetadata>,
  /// Configuration for when wick hosts this component as a service.

  #[serde(default)]
  pub(crate) host: HostConfig,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
pub(crate) enum ComponentKind {
  /// A variant representing a [WasmComponentConfiguration] type.
  #[serde(rename = "wick/component/wasmrs@v1")]
  WasmComponentConfiguration(WasmComponentConfiguration),
  /// A variant representing a [CompositeComponentConfiguration] type.
  #[serde(rename = "wick/component/composite@v1")]
  CompositeComponentConfiguration(CompositeComponentConfiguration),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component made out of other components
pub(crate) struct CompositeComponentConfiguration {
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<wick_interface_types::TypeDefinition>,
  /// Components to import into the application&#x27;s scope.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) import: Vec<ComponentBinding>,
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

  #[serde(deserialize_with = "with_expand_envs")]
  #[serde(rename = "ref")]
  pub(crate) reference: String,
  /// Additional types to export and make available to the component.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) types: Vec<wick_interface_types::TypeDefinition>,
  /// A list of operations implemented by the WebAssembly module.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<OperationDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Metadata for the component.
pub(crate) struct ComponentMetadata {
  /// The version of the component.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a resource.
pub(crate) struct ResourceBinding {
  /// The name of the binding.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// The resource to bind to.
  pub(crate) resource: ResourceDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a component.
pub(crate) struct ComponentBinding {
  /// The name of the binding.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// The component to bind to.

  #[serde(deserialize_with = "crate::v1::parse::component_shortform")]
  pub(crate) component: ComponentDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The possible types of components.
pub(crate) enum ComponentDefinition {
  /// A variant representing a [WasmRsComponent] type.
  #[serde(rename = "wick/component/wasmrs@v1")]
  WasmRsComponent(WasmRsComponent),
  /// A variant representing a [GrpcUrlComponent] type.
  #[serde(rename = "wick/component/grpc@v1")]
  GrpcUrlComponent(GrpcUrlComponent),
  /// A variant representing a [ManifestComponent] type.
  #[serde(rename = "wick/component/manifest@v1")]
  ManifestComponent(ManifestComponent),
  /// A variant representing a [ComponentReference] type.
  #[serde(rename = "wick/component/reference@v1")]
  ComponentReference(ComponentReference),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A reference to a component in the application's scope.
pub(crate) struct ComponentReference {
  /// The id of the component to reference.

  #[serde(deserialize_with = "with_expand_envs")]
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
  pub(crate) port: Option<u16>,
  /// The address to bind to.

  #[serde(default)]
  pub(crate) address: Option<String>,
  /// Path to pem file for TLS.

  #[serde(default)]
  pub(crate) pem: Option<String>,
  /// Path to key file for TLS.

  #[serde(default)]
  pub(crate) key: Option<String>,
  /// Path to CA file.

  #[serde(default)]
  pub(crate) ca: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A WebAssembly component.
pub(crate) struct WasmRsComponent {
  /// The URL (and optional tag) or local file path to find the .wasm module.

  #[serde(deserialize_with = "with_expand_envs")]
  #[serde(rename = "ref")]
  pub(crate) reference: String,
  /// Permissions to give this component

  #[serde(default)]
  pub(crate) permissions: Permissions,
  /// Per-component configuration.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Per-component permissions configuration.
pub(crate) struct Permissions {
  /// A map of from internal directory to external directory that this component should be able to access.

  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub(crate) dirs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component hosted as an independent microservice.
pub(crate) struct GrpcUrlComponent {
  /// The GRPC URL to connect to.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) url: String,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A native component that can be extracted and run as a microservice.
pub(crate) struct ManifestComponent {
  /// The URL (and optional tag) or local file path to find the manifest.

  #[serde(deserialize_with = "with_expand_envs")]
  #[serde(rename = "ref")]
  pub(crate) reference: String,
  /// Any configuration necessary for the component.

  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A definition for an single composite operation.
pub(crate) struct CompositeOperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<wick_interface_types::Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<wick_interface_types::Field>,
  /// A list of components the schematic can use.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) components: Vec<String>,
  /// A map of IDs to specific operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) instances: Vec<InstanceBinding>,
  /// A list of connections from operation to operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::v1::parse::vec_connection")]
  pub(crate) flow: Vec<ConnectionDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An operation name and its input and output signatures
pub(crate) struct OperationDefinition {
  /// The name of the operation.

  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// Types of the inputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<wick_interface_types::Field>,
  /// Types of the outputs to the operation.

  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<wick_interface_types::Field>,
}

/// Field definition. This is not technically an any type, it is a wick interface type field.
#[allow(unused)]
pub(crate) type Field = Value;

/// Type definition. This is not technically an any type, it is a wick interface type definition.
#[allow(unused)]
pub(crate) type TypeDefinition = Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// An identifier bound to a component's operation.
pub(crate) struct InstanceBinding {
  /// The name of the binding.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// The operation to bind to.

  #[serde(deserialize_with = "crate::v1::parse::component_operation_syntax")]
  pub(crate) operation: ComponentOperationExpression,
  /// Data to associate with the reference, if any.

  #[serde(default)]
  pub(crate) config: Option<Value>,
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

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) instance: String,
  /// The operation port.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: String,
  /// The default value to provide on this connection in the event of an error.

  #[serde(default)]
  pub(crate) data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A test case for a component.
pub(crate) struct TestDefinition {
  /// The name of the test.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// The operaton to test.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) operation: String,
  /// Inherent data to use for the test.

  #[serde(default)]
  pub(crate) inherent: Option<InherentData>,
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
  pub(crate) seed: Option<u64>,
  /// A timestamp.

  #[serde(default)]
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

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// Any flags set on the packet.

  #[serde(default)]
  pub(crate) flags: Option<PacketFlags>,
  /// The data to send.

  #[serde(default)]
  pub(crate) data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ErrorData {
  /// The name of the port to send the data to.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// Any flags set on the packet.

  #[serde(default)]
  pub(crate) flags: Option<PacketFlags>,
  /// The error message.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) message: String,
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
