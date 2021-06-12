use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vino_manifest::NetworkManifest;

use crate::schematic_definition::SchematicDefinition;

#[derive(Debug, Clone)]
pub struct NetworkDefinition {
    pub schematics: Vec<SchematicDefinition>,
}

impl NetworkDefinition {
    pub fn new(manifest: &NetworkManifest) -> Self {
        match manifest {
            NetworkManifest::V0(manifest) => Self {
                schematics: manifest
                    .schematics
                    .clone()
                    .into_iter()
                    .map(|val| val.into())
                    .collect(),
            },
        }
    }
}

impl From<vino_manifest::v0::NetworkManifest> for NetworkDefinition {
    fn from(def: vino_manifest::v0::NetworkManifest) -> Self {
        Self::new(&vino_manifest::NetworkManifest::V0(def))
    }
}

impl Default for NetworkDefinition {
    fn default() -> Self {
        Self { schematics: vec![] }
    }
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
