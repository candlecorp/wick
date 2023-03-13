use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use tracing::debug;

use crate::error::ManifestError;
use crate::host_definition::HostConfig;
use crate::v1::ComponentMetadata;
use crate::{from_yaml, v0, v1, ComponentDefinition, Error, FlowOperation, Result};

#[derive(Debug, Clone, Default)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  source: Option<String>,
  format: u32,
  version: String,
  host: HostConfig,
  name: Option<String>,
  labels: HashMap<String, String>,
  import: HashMap<String, ComponentDefinition>,
  operations: HashMap<String, FlowOperation>,
}

impl TryFrom<v0::HostManifest> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v0::HostManifest) -> Result<Self> {
    let flows: Result<HashMap<String, FlowOperation>> = def
      .network
      .schematics
      .iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    Ok(ComponentConfiguration {
      source: None,
      format: def.format,
      version: def.version,
      host: def.host.try_into()?,
      name: def.network.name,
      import: def
        .network
        .collections
        .iter()
        .map(|val| Ok((val.namespace.clone(), val.try_into()?)))
        .collect::<Result<HashMap<_, _>>>()?,
      labels: def.network.labels,
      operations: flows?,
    })
  }
}

impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      format: def.format,
      version: def.metadata.unwrap_or(ComponentMetadata::default()).version,
      host: def.host.try_into()?,
      name: def.name,
      import: def
        .import
        .into_iter()
        .map(|(k, v)| (k.clone(), (k, v).into()))
        .collect(),
      labels: def.labels,
      operations: def
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
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
    let raw_version = raw.get("format").ok_or(Error::NoFormat)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => Ok(from_yaml::<v0::HostManifest>(src)?.try_into()?),
      1 => Ok(from_yaml::<v1::ComponentConfiguration>(src)?.try_into()?),
      -1 => Err(Error::NoFormat),
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
  pub fn format(&self) -> u32 {
    self.format
  }

  /// Return the version of the component.
  #[must_use]
  pub fn version(&self) -> &str {
    &self.version
  }

  /// Return the underlying version of the source manifest.
  #[must_use]
  pub fn source(&self) -> &Option<String> {
    &self.source
  }

  #[must_use]
  /// Get a map of [Flow]s from the [ComponentConfiguration]
  pub fn operations(&self) -> &HashMap<String, FlowOperation> {
    &self.operations
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
  pub fn components(&self) -> &HashMap<String, ComponentDefinition> {
    &self.import
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn component(&self, namespace: &str) -> Option<&ComponentDefinition> {
    self.import.iter().find(|(k, _)| *k == namespace).map(|(_, v)| v)
  }

  /// Get a schematic by name
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&FlowOperation> {
    self.operations.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }
}

/// ComponentConfiguration builder.
#[derive(Default, Debug, Clone)]
#[must_use]
pub struct ComponentConfigurationBuilder {
  version: Option<String>,
  base: Option<ComponentConfiguration>,
  components: HashMap<String, ComponentDefinition>,
  flows: HashMap<String, FlowOperation>,
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

  /// Set the version of the component.
  pub fn version(mut self, version: impl AsRef<str>) -> Self {
    self.version = Some(version.as_ref().to_owned());
    self
  }

  /// Add a [CollectionDefinition] to the builder.
  pub fn add_collection(mut self, name: impl AsRef<str>, collection: ComponentDefinition) -> Self {
    self.components.insert(name.as_ref().to_owned(), collection);
    self
  }

  /// Add a [Flow] to the builder.
  pub fn add_flow(mut self, name: impl AsRef<str>, flow: FlowOperation) -> Self {
    self.flows.insert(name.as_ref().to_owned(), flow);
    self
  }

  /// Consume the [ComponentConfigurationBuilder] and return a [ComponentConfiguration].
  pub fn build(self) -> ComponentConfiguration {
    if let Some(mut def) = self.base {
      for (name, collection) in self.components {
        def.import.insert(name, collection);
      }
      for (name, flow) in self.flows {
        def.operations.insert(name, flow);
      }
      def
    } else {
      ComponentConfiguration {
        version: self.version.unwrap_or("0.0.0".to_owned()),
        import: self.components,
        operations: self.flows,
        ..Default::default()
      }
    }
  }
}
