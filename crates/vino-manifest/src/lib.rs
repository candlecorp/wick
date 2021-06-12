use std::{fs::read_to_string, path::Path};

use hocon::HoconLoader;
use log::debug;

pub mod error;
pub mod v0;
pub type Error = error::ManifestError;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum HostManifest {
    V0(v0::HostManifest),
}

#[derive(Debug, Clone)]
pub enum NetworkManifest {
    V0(v0::NetworkManifest),
}

#[derive(Debug, Clone)]
pub enum SchematicManifest {
    V0(v0::SchematicManifest),
}

impl HostManifest {
    pub fn load_from_file(path: &Path) -> Result<HostManifest> {
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_string_lossy().into()));
        }
        debug!("Reading manifest from {}", path.to_string_lossy());
        let contents = read_to_string(path)?;
        Self::from_hocon(&contents).or_else(|e| {
            debug!("{:?}", e);
            match &e {
                Error::VersionError(_) => Err(e),
                Error::HoconError(hocon::Error::Deserialization { message }) => {
                    // Hocon doesn't differentiate errors for disallowed fields and a bad parse
                    if message.contains("unknown field") {
                        debug!("Invalid field found in hocon");
                        Err(e)
                    } else {
                        Self::from_yaml(&contents)
                    }
                }
                _ => Self::from_yaml(&contents),
            }
        })
    }
    pub fn from_hocon(src: &str) -> Result<HostManifest> {
        debug!("Trying to parse manifest as hocon");
        let raw = HoconLoader::new().strict().load_str(src)?.hocon()?;

        let a = raw["version"]
            .as_string()
            .unwrap_or_else(|| "Version not found".into());
        match a.as_str() {
            "0" => Ok(HostManifest::V0(hocon::de::from_str(src)?)),
            _ => Err(Error::VersionError(a.to_string())),
        }
    }
    pub fn from_yaml(src: &str) -> Result<HostManifest> {
        debug!("Trying to parse manifest as yaml");
        let raw: serde_yaml::Value = serde_yaml::from_str(src)?;

        let a = raw["version"].as_str().unwrap_or("Version not found");
        match a {
            "0" => Ok(HostManifest::V0(serde_yaml::from_str(src)?)),
            _ => Err(Error::VersionError(a.to_string())),
        }
    }
}
