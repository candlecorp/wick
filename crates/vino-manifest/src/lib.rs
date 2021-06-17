//! Vino Manifest implementation

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

use std::{fs::read_to_string, path::Path};

use hocon::HoconLoader;
use log::debug;

/// Vino Manifest error
pub mod error;

/// Version 0 (unstable) manifest
pub mod v0;

/// The crate's error type
pub type Error = error::ManifestError;

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Enum for the possible versions of a Host Manifest
#[derive(Debug, Clone)]
pub enum HostManifest {
  /// Version 0 Host Manifest
  V0(v0::HostManifest),
}

/// Enum for the possible versions of a Network Manifest
#[derive(Debug, Clone)]
pub enum NetworkManifest {
  /// Version 0 Network Manifest
  V0(v0::NetworkManifest),
}

/// Enum for the possible versions of a Schematic Manifest
#[derive(Debug, Clone)]
pub enum SchematicManifest {
  /// Version 0 Schematic Manifest
  V0(v0::SchematicManifest),
}

impl HostManifest {
  /// Load a manifest from a file
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

  pub(crate) fn from_hocon(src: &str) -> Result<HostManifest> {
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

  pub(crate) fn from_yaml(src: &str) -> Result<HostManifest> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = serde_yaml::from_str(src)?;

    let a = raw["version"].as_str().unwrap_or("Version not found");
    match a {
      "0" => Ok(HostManifest::V0(serde_yaml::from_str(src)?)),
      _ => Err(Error::VersionError(a.to_string())),
    }
  }
}
