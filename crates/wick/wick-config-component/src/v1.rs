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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with_expand_env::with_expand_envs;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A manifest defines the starting state of a Wick host and network.
pub struct ComponentConfiguration {
  /// The manifest version.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub version: u8,
  /// Configuration for the host when this manifest is run directly.
  #[serde(default)]
  pub host: HostConfig,
  /// The default flow to execute if none is provided.
  #[serde(default)]
  pub default_flow: Option<String>,
  /// The unique identifier for this manifest.
  #[serde(default)]
  pub name: Option<String>,
  /// The labels and values that apply to this manifest.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub labels: HashMap<String, String>,
  /// A map of namespace to external components.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::parse::v1::component_shortform")]
  pub external: HashMap<String, ComponentDefinition>,
  /// A map of operation names to their definitions.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub operations: HashMap<String, FlowDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
/// The possible types of components.
pub enum ComponentDefinition {
  /// A variant representing a [WasmComponent] type.
  #[serde(rename = "Wasm")]
  WasmComponent(WasmComponent),
  /// A variant representing a [GrpcUrlComponent] type.
  #[serde(rename = "GrpcUrl")]
  GrpcUrlComponent(GrpcUrlComponent),
  /// A variant representing a [GrpcTarComponent] type.
  #[serde(rename = "GrpcTar")]
  GrpcTarComponent(GrpcTarComponent),
  /// A variant representing a [MeshComponent] type.
  #[serde(rename = "Mesh")]
  MeshComponent(MeshComponent),
  /// A variant representing a [ManifestComponent] type.
  #[serde(rename = "Manifest")]
  ManifestComponent(ManifestComponent),
}

#[allow(non_snake_case)]
pub(crate) fn HOST_CONFIG_TIMEOUT() -> u64 {
  5000
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Host configuration options.
pub struct HostConfig {
  /// Whether or not to allow the :latest tag on remote artifacts.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub allow_latest: bool,
  /// A list of registries to connect to insecurely (over HTTP vs HTTPS).
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub insecure_registries: Vec<String>,
  /// The timeout for network requests (in ms).
  #[serde(default = "HOST_CONFIG_TIMEOUT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub timeout: u64,
  /// The ID for this host, used to identify the host over the mesh.
  #[serde(default)]
  pub id: Option<String>,
  /// The schematics to expose via RPC or the mesh, if any.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub expose: Vec<String>,
  /// The mesh configuration.
  #[serde(default)]
  pub mesh: Option<MeshConfig>,
  /// Configuration for the GRPC server.
  #[serde(default)]
  pub rpc: Option<HttpConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for the GRPC service.
pub struct HttpConfig {
  /// Enable/disable the server.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub enabled: bool,
  /// The port to bind to.
  #[serde(default)]
  pub port: Option<u16>,
  /// The address to bind to.
  #[serde(default)]
  pub address: Option<String>,
  /// Path to pem file for TLS.
  #[serde(default)]
  pub pem: Option<String>,
  /// Path to key file for TLS.
  #[serde(default)]
  pub key: Option<String>,
  /// Path to CA file.
  #[serde(default)]
  pub ca: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration used to connect to the mesh.
pub struct MeshConfig {
  /// Enable/disable the mesh connection.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub enabled: bool,
  /// The address of the NATS server.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub address: String,
  /// The path to the NATS credsfile.
  #[serde(default)]
  pub creds_path: Option<String>,
  /// The NATS token.
  #[serde(default)]
  pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A WebAssembly component.
pub struct WasmComponent {
  /// The URL (and optional tag) or local file path to find the .wasm module.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub reference: String,
  /// Permissions to give this component
  #[serde(default)]
  pub permissions: Permissions,
  /// Per-component configuration.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Per-component permissions configuration.
pub struct Permissions {
  /// A map of from internal directory to external directory that this component should be able to access.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::helpers::kv_deserializer")]
  pub dirs: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component hosted as an independent microservice.
pub struct GrpcUrlComponent {
  /// The GRPC URL to connect to.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub url: String,
  /// Any configuration necessary for the component.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A component hosted somewhere on a connected mesh.
pub struct MeshComponent {
  /// The ID of the component.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub id: String,
  /// Any configuration necessary for the component.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A native component that can be extracted and run as a microservice.
pub struct GrpcTarComponent {
  /// The URL (and optional tag) or local file path to find the archive.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub reference: String,
  /// Any configuration necessary for the component.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A native component that can be extracted and run as a microservice.
pub struct ManifestComponent {
  /// The URL (and optional tag) or local file path to find the manifest.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub reference: String,
  /// Any configuration necessary for the component.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub config: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A definition for an single flow.
pub struct FlowDefinition {
  /// A list of components the schematic can use.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// A map of IDs to specific operation.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "crate::parse::v1::map_component_def")]
  pub instances: HashMap<String, InstanceDefinition>,
  /// A list of connections from operation to operation.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "crate::parse::v1::vec_connection")]
  pub flow: Vec<ConnectionDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// The ID and configuration for an operation.
pub struct InstanceDefinition {
  /// The ID to assign to this instance of the operation.
  #[serde(deserialize_with = "with_expand_envs")]
  pub id: String,
  /// Data to associate with the reference.
  #[serde(default)]
  pub config: Option<Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection between Operations and their ports. This can be specified in short-form syntax (where applicable). See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information.
pub struct ConnectionDefinition {
  /// The upstream operation port.
  #[serde(default)]
  #[serde(deserialize_with = "crate::parse::v1::connection_target_shortform")]
  pub from: ConnectionTargetDefinition,
  /// The downstream operation port.
  #[serde(default)]
  #[serde(deserialize_with = "crate::parse::v1::connection_target_shortform")]
  pub to: ConnectionTargetDefinition,
  /// The default value to provide in the event of an upstream Error or Exception.
  #[serde(default)]
  pub default: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable).  See <a href='https://wasmflow.com/docs/configuration/short-form-syntax/'>wasmflow.com</a> for more information.
pub struct ConnectionTargetDefinition {
  /// The instance ID of the operation.
  #[serde(deserialize_with = "with_expand_envs")]
  pub instance: String,
  /// The operation port.
  #[serde(deserialize_with = "with_expand_envs")]
  pub port: String,
  /// The default value to provide on this connection in the event of an error.
  #[serde(default)]
  pub data: Option<Value>,
}
