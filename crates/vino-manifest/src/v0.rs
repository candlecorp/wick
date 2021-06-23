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

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use serde::{
  Deserialize,
  Serialize,
};

#[allow(non_snake_case)]
fn HOST_MANIFEST_DEFAULT_SCHEMATIC() -> String {
  "default".to_string()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// The Host Manifest defines the starting state of a Vino host
pub struct HostManifest {
  /// The manifest version
  pub version: String,
  /// The configuration for a Vino network
  #[serde(default)]
  pub network: NetworkManifest,
  /// The default schematic to execute if none is provided
  #[serde(default = "HOST_MANIFEST_DEFAULT_SCHEMATIC")]
  pub default_schematic: String,
  /// The NATS configuration to connect hosts
  #[serde(default)]
  pub nats: NatsConfiguration,
}

#[allow(non_snake_case)]
fn NATS_CONFIGURATION_RPC_HOST() -> String {
  "0.0.0.0".to_string()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_RPC_PORT() -> String {
  "4222".to_string()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_CONTROL_HOST() -> String {
  "0.0.0.0".to_string()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_CONTROL_PORT() -> String {
  "4222".to_string()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// The NATS configuration
pub struct NatsConfiguration {
  /// The host for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_RPC_HOST")]
  pub rpc_host: String,
  /// The port for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_RPC_PORT")]
  pub rpc_port: String,
  /// The NATS credsfile for the RPC layer
  #[serde(default)]
  pub rpc_credsfile: Option<String>,
  /// The JSON web token to use for authentication to the RPC layer
  #[serde(default)]
  pub rpc_jwt: Option<String>,
  /// The seed to use for authentication to the RPC layer
  #[serde(default)]
  pub rpc_seed: Option<String>,
  /// The host for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_CONTROL_HOST")]
  pub control_host: String,
  /// The port for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_CONTROL_PORT")]
  pub control_port: String,
  /// The NATS credsfile for the RPC layer
  #[serde(default)]
  pub control_credsfile: Option<String>,
  /// The JSON web token to use for authentication to the RPC layer
  #[serde(default)]
  pub control_jwt: Option<String>,
  /// The seed to use for authentication to the RPC layer
  #[serde(default)]
  pub control_seed: Option<String>,
  /// Enable :latest tags for OCI references
  #[serde(default)]
  pub allow_oci_latest: bool,
  /// A list of insecure registries to allow
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub allowed_insecure: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A Vino network definition
pub struct NetworkManifest {
  /// The labels that apply to this host
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub labels: HashMap<String, String>,
  /// The links between capabilities and components
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub schematics: Vec<SchematicManifest>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub capabilities: Vec<Capability>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<LinkEntry>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub actors: Vec<String>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub components: Vec<String>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub references: Vec<ReferenceEntry>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub connections: Vec<ConnectionEntry>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// deprecated
pub struct Capability {
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub image_ref: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub link_name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// deprecated
pub struct LinkEntry {
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub actor: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub contract_id: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub provider_id: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub link_name: Option<String>,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub values: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// deprecated
pub struct ReferenceEntry {
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub reference: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub target: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// deprecated
pub struct ConnectionEntry {
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub from: ActorPortEntry,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub to: ActorPortEntry,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// deprecated
pub struct ActorPortEntry {
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub reference: String,
  /// deprecated
  #[deprecated()]
  #[serde(default)]
  pub port: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A definition for an individual Vino schematic
pub struct SchematicManifest {
  /// Schematic name
  #[serde(default)]
  pub name: String,
  /// A list of external components
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub external: Vec<ExternalComponentDefinition>,
  /// A list of providers and component collections
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub providers: Vec<ProviderDefinition>,
  /// A map from component reference to its target
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub components: HashMap<String, ComponentDefinition>,
  /// A list of connections from component to component
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub connections: Vec<ConnectionDefinition>,
  /// A map of constraints and values that limit where this schematic can run
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub constraints: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// An external component definition
pub struct ExternalComponentDefinition {
  /// An alias to use for this component (local to this manifest only)
  #[serde(default)]
  pub alias: Option<String>,
  /// The location reference (i.e. URL or file path)
  pub reference: String,
  /// A public key to verify the retrieved component&#x27;s validity
  #[serde(default)]
  pub key: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A provider definition
pub struct ProviderDefinition {
  /// The namespace to reference the provider&#x27;s components on
  #[serde(default)]
  pub namespace: String,
  /// The kind/type of the provider
  #[serde(default)]
  pub kind: ProviderKind,
  /// The reference/location of the provider
  #[serde(default)]
  pub reference: String,
  /// Data or configuration to pass to the provider initialization
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Eq, PartialEq, Primitive)]
#[serde(deny_unknown_fields)]
/// Kind of provider,
pub enum ProviderKind {
  /// Native providers included at compile-time in a Vino host
  Native = 0,
  /// The URL for a separately managed GRPC endpoint
  GrpcUrl = 1,
}

impl Default for ProviderKind {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A single component definition
pub struct ComponentDefinition {
  /// The ID of the component (i.e. the alias, key, or namespace)
  pub id: String,
  /// Unused (reserved)
  #[serde(default)]
  pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A connection between components
pub struct ConnectionDefinition {
  /// The originating component (upstream)
  pub from: ConnectionTargetDefinition,
  /// The destination component (downstream)
  pub to: ConnectionTargetDefinition,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A connection target
pub struct ConnectionTargetDefinition {
  /// The component reference
  pub instance: String,
  /// The component&#x27;s port
  pub port: String,
}