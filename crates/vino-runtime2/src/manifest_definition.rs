use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::SchematicDefinition;

/// A host manifest contains a declarative profile of the host's desired state. The manifest
/// can specify custom labels, a list of actors, a list of capability providers, and a list of
/// link definitions. Environment substitution syntax can optionally be used within a manifest file so that
/// information that may change across environments (like public keys) can change without requiring
/// the manifest file to change.
///
/// # Examples
///
/// ```yaml
/// labels:
///     sample: "wasmcloud echo"
/// actors:
///     - "wasmcloud.azurecr.io/echo:0.2.0"
/// capabilities:
///     - image_ref: wasmcloud.azurecr.io/httpserver:0.11.1
///       link_name: default
/// links:
///     - actor: ${ECHO_ACTOR:MBCFOPM6JW2APJLXJD3Z5O4CN7CPYJ2B4FTKLJUR5YR5MITIU7HD3WD5}
///       provider_id: "VAG3QITQQ2ODAOWB5TTQSDJ53XK3SHBEIFNK4AYJ5RKAX2UNSCAPHA5M"
///       contract_id: "wasmcloud:httpserver"
///       link_name: default
///       values:
///         PORT: 8080
/// ```

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct HostManifest {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[doc(hidden)]
    pub labels: HashMap<String, String>,
    #[doc(hidden)]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actors: Vec<String>,
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
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceEntry>,
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
