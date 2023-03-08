use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use tracing::debug;

use crate::error::ManifestError;
use crate::host_definition::HostConfig;
use crate::{from_yaml, v0, v1, ComponentDefinition, Error, Flow, Result};

#[derive(Debug, Clone, Default)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  source: Option<String>,
  version: u8,
  host: HostConfig,
  default_flow: Option<String>,
  name: Option<String>,
  labels: HashMap<String, String>,
  collections: HashMap<String, ComponentDefinition>,
  flows: HashMap<String, Flow>,
}

impl TryFrom<v0::HostManifest> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v0::HostManifest) -> Result<Self> {
    let flows: Result<HashMap<String, Flow>> = def
      .network
      .schematics
      .iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    Ok(ComponentConfiguration {
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

impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
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
        .operations
        .into_iter()
        .map(|(k, v)| Ok((k.clone(), (k, v).try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl ComponentConfiguration {
  /// Load struct from file by trying all the supported file formats.
  pub fn load_from_file(path: impl AsRef<Path>) -> Result<ComponentConfiguration> {
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
  pub fn load_from_bytes(source: Option<String>, bytes: &[u8]) -> Result<ComponentConfiguration> {
    let contents = String::from_utf8_lossy(bytes);
    let mut manifest = Self::from_yaml(&contents)?;
    manifest.source = source;
    Ok(manifest)
  }

  /// Load as YAML.
  pub fn from_yaml(src: &str) -> Result<ComponentConfiguration> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src)?;
    debug!("Yaml parsed successfully");
    let raw_version = raw.get("version").ok_or(Error::NoVersion)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => Ok(from_yaml::<v0::HostManifest>(src)?.try_into()?),
      1 => Ok(from_yaml::<v1::ComponentConfiguration>(src)?.try_into()?),
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
  /// Get a map of [Flow]s from the [ComponentConfiguration]
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
  pub fn collections(&self) -> &HashMap<String, ComponentDefinition> {
    &self.collections
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn collection(&self, namespace: &str) -> Option<&ComponentDefinition> {
    self.collections.iter().find(|(k, _)| *k == namespace).map(|(_, v)| v)
  }

  /// Get a schematic by name
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&Flow> {
    self.flows.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }
}

/// ComponentConfiguration builder.
#[derive(Default, Debug, Clone)]
#[must_use]
pub struct ComponentConfigurationBuilder {
  base: Option<ComponentConfiguration>,
  collections: HashMap<String, ComponentDefinition>,
  flows: HashMap<String, Flow>,
}

impl ComponentConfigurationBuilder {
  /// Create a new [ComponentConfigurationBuilder].
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a builder with an existing manifest as a base.
  pub fn with_base(definition: ComponentConfiguration) -> Self {
    Self {
      base: Some(definition),
      ..Default::default()
    }
  }

  /// Add a [CollectionDefinition] to the builder.
  pub fn add_collection(mut self, name: impl AsRef<str>, collection: ComponentDefinition) -> Self {
    self.collections.insert(name.as_ref().to_owned(), collection);
    self
  }

  /// Add a [Flow] to the builder.
  pub fn add_flow(mut self, name: impl AsRef<str>, flow: Flow) -> Self {
    self.flows.insert(name.as_ref().to_owned(), flow);
    self
  }

  /// Consume the [ComponentConfigurationBuilder] and return a [ComponentConfiguration].
  pub fn build(self) -> ComponentConfiguration {
    if let Some(mut def) = self.base {
      for (name, collection) in self.collections {
        def.collections.insert(name, collection);
      }
      for (name, flow) in self.flows {
        def.flows.insert(name, flow);
      }
      def
    } else {
      ComponentConfiguration {
        version: 1,
        collections: self.collections,
        flows: self.flows,
        ..Default::default()
      }
    }
  }
}
