use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(non_snake_case)]
fn HOST_MANIFEST_DEFAULT_SCHEMATIC() -> String {
    "default".to_string()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostManifest {
    pub version: String,
    #[serde(default)]
    pub network: NetworkManifest,
    #[serde(default = "HOST_MANIFEST_DEFAULT_SCHEMATIC")]
    pub default_schematic: String,
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
pub struct NatsConfiguration {
    #[serde(default = "NATS_CONFIGURATION_RPC_HOST")]
    pub rpc_host: String,
    #[serde(default = "NATS_CONFIGURATION_RPC_PORT")]
    pub rpc_port: String,
    #[serde(default)]
    pub rpc_credsfile: Option<String>,
    #[serde(default)]
    pub rpc_jwt: Option<String>,
    #[serde(default)]
    pub rpc_seed: Option<String>,
    #[serde(default = "NATS_CONFIGURATION_CONTROL_HOST")]
    pub control_host: String,
    #[serde(default = "NATS_CONFIGURATION_CONTROL_PORT")]
    pub control_port: String,
    #[serde(default)]
    pub control_credsfile: Option<String>,
    #[serde(default)]
    pub control_jwt: Option<String>,
    #[serde(default)]
    pub control_seed: Option<String>,
    #[serde(default)]
    pub allow_oci_latest: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_insecure: std::vec::Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkManifest {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: std::collections::HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: std::vec::Vec<Capability>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: std::vec::Vec<LinkEntry>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schematics: std::vec::Vec<SchematicManifest>,
    #[deprecated()]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actors: std::vec::Vec<String>,
    #[deprecated()]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: std::vec::Vec<String>,
    #[deprecated()]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: std::vec::Vec<ReferenceEntry>,
    #[deprecated()]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connections: std::vec::Vec<ConnectionEntry>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    #[serde(default)]
    pub image_ref: String,
    #[serde(default)]
    pub link_name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkEntry {
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub contract_id: String,
    #[serde(default)]
    pub provider_id: String,
    #[serde(default)]
    pub link_name: Option<String>,
    #[serde(default)]
    pub values: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceEntry {
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub target: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionEntry {
    #[serde(default)]
    pub from: ActorPortEntry,
    #[serde(default)]
    pub to: ActorPortEntry,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorPortEntry {
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub port: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchematicManifest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external: std::vec::Vec<ExternalComponentDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: std::collections::HashMap<String, ComponentDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connections: std::vec::Vec<ConnectionDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub constraints: std::collections::HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalComponentDefinition {
    #[serde(default)]
    pub alias: Option<String>,
    pub reference: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentDefinition {
    #[serde(default)]
    pub metadata: Option<String>,
    pub id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionDefinition {
    pub from: ConnectionTargetDefinition,
    pub to: ConnectionTargetDefinition,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionTargetDefinition {
    pub instance: String,
    pub port: String,
}
