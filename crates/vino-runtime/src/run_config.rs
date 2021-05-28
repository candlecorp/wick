use serde::{Deserialize, Serialize};
use wasmcloud_host::HostManifest;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunConfig {
    pub manifest: HostManifest,

    #[serde(default = "DEFAULT_SCHEMATIC")]
    pub default_schematic: String,

    #[serde(default)]
    pub config: CommonConfiguration,
}

#[allow(non_snake_case)]
fn DEFAULT_SCHEMATIC() -> String {
    "default".to_string()
}

#[allow(non_snake_case)]
fn DEFAULT_RPC_HOST() -> String {
    "0.0.0.0".to_string()
}

#[allow(non_snake_case)]
fn DEFAULT_RPC_PORT() -> String {
    "4222".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonConfiguration {
    #[serde(default = "DEFAULT_RPC_HOST")]
    pub rpc_host: String,

    #[serde(default = "DEFAULT_RPC_PORT")]
    pub rpc_port: String,

    #[serde(default)]
    pub rpc_credentials: Option<String>,

    #[serde(default)]
    pub control_credentials: Option<String>,

    #[serde(default)]
    pub allow_oci_latest: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_insecure: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NatsConfiguration {
    #[serde(default = "DEFAULT_RPC_HOST")]
    pub rpc_host: String,

    #[serde(default = "DEFAULT_RPC_PORT")]
    pub rpc_port: String,

    #[serde(default)]
    pub rpc_credentials: Option<String>,

    #[serde(default)]
    pub control_credentials: Option<String>,

    #[serde(default)]
    pub allow_oci_latest: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_insecure: Vec<String>,
}
