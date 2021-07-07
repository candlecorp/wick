//! Vino Manifest implementation

// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
    clippy::expect_used,
    clippy::explicit_deref_methods,
    clippy::option_if_let_else,
    clippy::await_holding_lock,
    clippy::cloned_instead_of_copied,
    clippy::explicit_into_iter_loop,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
    deprecated,
    explicit_outlives_requirements,
    improper_ctypes,
    invalid_value,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

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
  parse_id,
  ComponentDefinition,
  ConnectionDefinition,
  ConnectionTargetDefinition,
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
      _ => Err(Error::VersionError(a.clone())),
    }
  }

  fn from_yaml(src: &str) -> Result<HostManifest> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src)?;

    let a = raw["version"].as_str().unwrap_or("Version not found");
    match a {
      "0" => Ok(HostManifest::V0(from_yaml(src)?)),
      _ => Err(Error::VersionError(a.to_owned())),
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
