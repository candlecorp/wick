use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::SchematicDefinition;

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct NetworkManifest {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[doc(hidden)]
    pub labels: HashMap<String, String>,
    #[deprecated]
    #[doc(hidden)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<String>,
    #[deprecated]
    #[doc(hidden)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[doc(hidden)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
    #[doc(hidden)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<LinkEntry>,
    #[deprecated]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceEntry>,
    #[deprecated]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<ConnectionEntry>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schematics: Vec<SchematicDefinition>,
}

/// The description of a capability within a host manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc(hidden)]
pub struct Capability {
    /// An image reference for this capability. If this is a file on disk, it will be used, otherwise
    /// the system will assume it is an OCI registry image reference
    pub image_ref: String,
    /// The (optional) name of the link that identifies this instance of the capability
    pub link_name: Option<String>,
}

/// A link definition describing the actor and capability provider involved, as well
/// as the configuration values for that link
#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc(hidden)]
pub struct LinkEntry {
    pub actor: String,
    pub contract_id: String,
    pub provider_id: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_name: Option<String>,
    pub values: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceEntry {
    pub reference: String,
    pub target: String,
}

/// A connection between two actor ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub from: ActorPortEntry,
    pub to: ActorPortEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorPortEntry {
    pub reference: String,
    pub port: String,
}
