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
  // missing_docs
)]
#![warn(clippy::cognitive_complexity)]

use std::fs::read_to_string;
use std::path::Path;

use hocon::HoconLoader;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tracing::debug;

/// Vino Manifest error
pub mod error;

/// Version 0 (unstable) manifest
pub mod v0;

/// A version-normalized format of the network manifest for development
pub mod network_definition;
/// A version-normalized format of the schematic manifest for development
pub mod schematic_definition;

pub use network_definition::NetworkDefinition;
pub use schematic_definition::{
  parse_namespace,
  ComponentDefinition,
  ConnectionDefinition,
  ConnectionTargetDefinition,
  ExternalComponentDefinition,
  ProviderDefinition,
  ProviderKind,
  SchematicDefinition,
};

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

trait DeserializeBorrowed<'de> {
  type Deserialize: Deserialize<'de>;
}

/// The Loadable trait can be used for any deserializable struct that can be loaded from
/// YAML, Hocon, or any other supported format.
pub trait Loadable<T> {
  /// Load struct from file by trying all the supported file formats
  fn load_from_file(path: &Path) -> Result<T> {
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
  /// Load as YAML
  fn from_yaml(src: &str) -> Result<T>;
  /// Load as Hocon
  fn from_hocon(src: &str) -> Result<T>;
}

fn from_yaml<T>(src: &str) -> Result<T>
where
  T: DeserializeOwned,
{
  let result = serde_yaml::from_str(src)?;
  debug!("Yaml parsed successfully");
  Ok(result)
}

fn from_hocon<T>(src: &str) -> Result<T>
where
  T: DeserializeOwned,
{
  let result = hocon::de::from_str(src).map_err(crate::Error::HoconError)?;
  debug!("Hocon parsed successfully");
  Ok(result)
}

impl Loadable<HostManifest> for HostManifest {
  fn from_hocon(src: &str) -> Result<HostManifest> {
    debug!("Trying to parse manifest as hocon");
    let raw = HoconLoader::new().strict().load_str(src)?.hocon()?;

    let a = raw["version"]
      .as_string()
      .unwrap_or_else(|| "Version not found".into());
    match a.as_str() {
      "0" => Ok(HostManifest::V0(from_hocon(src)?)),
      _ => Err(Error::VersionError(a.to_string())),
    }
  }

  fn from_yaml(src: &str) -> Result<HostManifest> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src)?;

    let a = raw["version"].as_str().unwrap_or("Version not found");
    match a {
      "0" => Ok(HostManifest::V0(from_yaml(src)?)),
      _ => Err(Error::VersionError(a.to_string())),
    }
  }
}

impl Loadable<v0::NetworkManifest> for v0::NetworkManifest {
  fn from_yaml(src: &str) -> Result<v0::NetworkManifest> {
    from_yaml(src)
  }

  fn from_hocon(src: &str) -> Result<v0::NetworkManifest> {
    from_hocon(src)
  }
}

impl Loadable<v0::SchematicManifest> for v0::SchematicManifest {
  fn from_yaml(src: &str) -> Result<v0::SchematicManifest> {
    from_yaml(src)
  }

  fn from_hocon(src: &str) -> Result<v0::SchematicManifest> {
    from_hocon(src)
  }
}
