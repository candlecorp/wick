use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
