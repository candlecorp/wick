//! Wasmflow Manifest implementation

// !!START_LINTS
// Wasmflow lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
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
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  dead_code,
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
  path_statements,
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
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq)]
// !!END_LINTS
// Add exceptions here
#![allow()]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use serde::de::DeserializeOwned;
use tracing::debug;

mod helpers;

/// Module for processing JSON templates used for default values.
mod default;
pub use default::{parse_default, process_default, ERROR_STR};

/// Module for parsing parts of a manifest.
pub(crate) mod parse;

/// Wasmflow Manifest error.
pub mod error;

/// Version 0 manifest.
pub mod v0;

/// Version 1 manifest.
pub mod v1;

/// A version-normalized format of the manifest for development.
pub mod host_definition;

/// A version-normalized format of the network manifest for development.
pub mod collection_definition;
pub use collection_definition::{CollectionDefinition, CollectionKind, Permissions};

/// A version-normalized format of the schematic manifest for development.
pub mod flow_definition;
pub use flow_definition::{ComponentDefinition, ConnectionDefinition, ConnectionTargetDefinition, Flow};
pub use wasmflow_parser::parse::v0::parse_id;

use self::host_definition::HostConfig;
use crate::error::ManifestError;

/// The crate's error type.
pub type Error = ManifestError;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Default)]
#[must_use]
/// The internal representation of a Wasmflow manifest.
pub struct WasmflowManifest {
  source: Option<String>,
  version: u8,
  host: HostConfig,
  default_flow: Option<String>,
  name: Option<String>,
  labels: HashMap<String, String>,
  collections: HashMap<String, CollectionDefinition>,
  flows: HashMap<String, Flow>,
}

impl TryFrom<v0::HostManifest> for WasmflowManifest {
  type Error = ManifestError;

  fn try_from(def: v0::HostManifest) -> Result<Self> {
    let flows: Result<HashMap<String, Flow>> = def
      .network
      .schematics
      .iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    Ok(WasmflowManifest {
      source: None,
      version: def.version,
      host: def.host.try_into()?,
      default_flow: def.default_schematic,
      name: def.network.name,
      collections: def
        .network
        .collections
        .iter()
        .map(|val| Ok((val.namespace.clone(), val.try_into()?)))
        .collect::<Result<HashMap<_, _>>>()?,
      labels: def.network.labels,
      flows: flows?,
    })
  }
}

impl TryFrom<v1::WasmflowManifest> for WasmflowManifest {
  type Error = ManifestError;

  fn try_from(def: v1::WasmflowManifest) -> Result<Self> {
    Ok(WasmflowManifest {
      source: None,
      version: def.version,
      host: def.host.try_into()?,
      default_flow: def.default_flow,
      name: def.name,
      collections: def
        .external
        .into_iter()
        .map(|(k, v)| (k.clone(), (k, v).into()))
        .collect(),
      labels: def.labels,
      flows: def
        .components
        .into_iter()
        .map(|(k, v)| Ok((k.clone(), (k, v).try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl WasmflowManifest {
  /// Load struct from file by trying all the supported file formats.
  pub fn load_from_file(path: impl AsRef<Path>) -> Result<WasmflowManifest> {
    let path = path.as_ref();
    if !path.exists() {
      return Err(Error::FileNotFound(path.to_string_lossy().into()));
    }
    debug!("Reading manifest from {}", path.to_string_lossy());
    let contents = read_to_string(path)?;
    let mut manifest = Self::from_yaml(&contents)?;
    manifest.source = Some(path.to_string_lossy().to_string());
    Ok(manifest)
  }

  /// Load struct from bytes by attempting to parse all the supported file formats.
  pub fn load_from_bytes(source: Option<String>, bytes: &[u8]) -> Result<WasmflowManifest> {
    let contents = String::from_utf8_lossy(bytes);
    let mut manifest = Self::from_yaml(&contents)?;
    manifest.source = source;
    Ok(manifest)
  }

  /// Load as YAML.
  pub fn from_yaml(src: &str) -> Result<WasmflowManifest> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src)?;
    debug!("Yaml parsed successfully");
    let raw_version = raw.get("version").ok_or(Error::NoVersion)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => Ok(from_yaml::<v0::HostManifest>(src)?.try_into()?),
      1 => Ok(from_yaml::<v1::WasmflowManifest>(src)?.try_into()?),
      -1 => Err(Error::NoVersion),
      _ => Err(Error::VersionError(version.to_string())),
    };

    debug!("Manifest: {:?}", manifest);
    manifest
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host(&self) -> &HostConfig {
    &self.host
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host_mut(&mut self) -> &mut HostConfig {
    &mut self.host
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  #[must_use]
  pub fn allow_latest(&self) -> bool {
    self.host.allow_latest
  }

  /// Return the list of insecure registries defined in the manifest
  #[must_use]
  pub fn insecure_registries(&self) -> &Vec<String> {
    &self.host.insecure_registries
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn version(&self) -> u8 {
    self.version
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> &Option<String> {
    &self.source
  }

  #[must_use]
  /// Get a map of [Flow]s from the [WasmflowManifest]
  pub fn flows(&self) -> &HashMap<String, Flow> {
    &self.flows
  }

  #[must_use]
  /// Get the default flow in this manifest.
  pub fn default_flow(&self) -> &Option<String> {
    &self.default_flow
  }

  /// Get the default flow in this manifest.
  pub fn set_default_flow(&mut self, name: impl AsRef<str>) {
    self.default_flow = Some(name.as_ref().to_owned());
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn labels(&self) -> &HashMap<String, String> {
    &self.labels
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn collections(&self) -> &HashMap<String, CollectionDefinition> {
    &self.collections
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn collection(&self, namespace: &str) -> Option<&CollectionDefinition> {
    self.collections.iter().find(|(k, _)| *k == namespace).map(|(_, v)| v)
  }

  /// Get a schematic by name
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&Flow> {
    self.flows.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }
}

/// WasmflowManifest builder.
#[derive(Default, Debug, Clone)]
#[must_use]
pub struct WasmflowManifestBuilder {
  base: Option<WasmflowManifest>,
  collections: HashMap<String, CollectionDefinition>,
  flows: HashMap<String, Flow>,
}

impl WasmflowManifestBuilder {
  /// Create a new [WasmflowManifestBuilder].
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a builder with an existing manifest as a base.
  pub fn with_base(definition: WasmflowManifest) -> Self {
    Self {
      base: Some(definition),
      ..Default::default()
    }
  }

  /// Add a [CollectionDefinition] to the builder.
  pub fn add_collection(mut self, name: impl AsRef<str>, collection: CollectionDefinition) -> Self {
    self.collections.insert(name.as_ref().to_owned(), collection);
    self
  }

  /// Add a [Flow] to the builder.
  pub fn add_flow(mut self, name: impl AsRef<str>, flow: Flow) -> Self {
    self.flows.insert(name.as_ref().to_owned(), flow);
    self
  }

  /// Consume the [WasmflowManifestBuilder] and return a [WasmflowManifest].
  pub fn build(self) -> WasmflowManifest {
    if let Some(mut def) = self.base {
      for (name, collection) in self.collections {
        def.collections.insert(name, collection);
      }
      for (name, flow) in self.flows {
        def.flows.insert(name, flow);
      }
      def
    } else {
      WasmflowManifest {
        version: 1,
        collections: self.collections,
        flows: self.flows,
        ..Default::default()
      }
    }
  }
}

fn from_yaml<T>(src: &str) -> Result<T>
where
  T: DeserializeOwned,
{
  let result = serde_yaml::from_str(src).map_err(|e| ManifestError::YamlError(e.to_string()))?;
  Ok(result)
}
