use std::{fs::read_to_string, path::Path};

use crate::{error::VinoHostError, Result};
use serde::{Deserialize, Serialize};
use vino_runtime::manifest::network_manifest::NetworkManifest;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostManifest {
    pub manifest: NetworkManifest,

    #[serde(default = "DEFAULT_SCHEMATIC")]
    pub default_schematic: String,

    #[serde(default)]
    pub config: CommonConfiguration,
}

impl HostManifest {
    pub fn load_from_file(path: &Path) -> Result<HostManifest> {
        ensure!(
            path.exists(),
            VinoHostError::FileNotFound(path.to_string_lossy().to_string()),
        );
        let contents = read_to_string(path)?;
        Self::from_hocon(&contents).or_else(|e| {
            debug!("Loading manifest as hocon failed: {}", e.to_string());
            Self::from_yaml(&contents)
        })
    }
    pub fn from_hocon(src: &str) -> Result<HostManifest> {
        debug!("Trying to parse manifest as hocon");
        Ok(hocon::de::from_str(src)?)
    }
    pub fn from_yaml(src: &str) -> Result<HostManifest> {
        debug!("Trying to parse manifest as yaml");
        Ok(serde_yaml::from_str(src)?)
    }
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
    pub rpc_credsfile: Option<String>,

    #[serde(default)]
    pub rpc_jwt: Option<String>,

    #[serde(default)]
    pub rpc_seed: Option<String>,

    #[serde(default = "DEFAULT_RPC_HOST")]
    pub control_host: String,

    #[serde(default = "DEFAULT_RPC_PORT")]
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
    pub allowed_insecure: Vec<String>,
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test_env_log::test(actix_rt::test)]
    async fn load_manifest_yaml() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("src");
        path.push("configurations");
        path.push("logger.yaml");
        let manifest = HostManifest::load_from_file(&path)?;

        assert_eq!(manifest.default_schematic, "logger");

        Ok(())
    }

    #[test_env_log::test(actix_rt::test)]
    async fn load_manifest_hocon() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("src");
        path.push("configurations");
        path.push("logger.manifest");
        let manifest = HostManifest::load_from_file(&path)?;

        assert_eq!(manifest.default_schematic, "logger");

        Ok(())
    }
}
