use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use tracing::debug;
use wick_interface_types::TypeDefinition;

use crate::component_definition::ComponentImplementation;
use crate::host_definition::HostConfig;
use crate::{from_yaml, v0, v1, BoundComponent, Error, FlowOperation, Result, TestCase};

#[derive(Debug, Clone)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  pub name: Option<String>,
  pub(crate) source: Option<String>,
  pub(crate) main: Option<ComponentImplementation>,
  pub(crate) format: u32,
  pub(crate) version: String,
  pub(crate) host: HostConfig,
  pub(crate) types: Vec<TypeDefinition>,
  pub(crate) labels: HashMap<String, String>,
  pub(crate) import: HashMap<String, BoundComponent>,
  pub(crate) operations: HashMap<String, FlowOperation>,
  pub(crate) tests: Vec<TestCase>,
}

impl Default for ComponentConfiguration {
  fn default() -> Self {
    Self {
      name: None,
      source: None,
      main: None,
      format: 1,
      version: "0.0.1".to_owned(),
      host: HostConfig::default(),
      types: vec![],
      labels: HashMap::new(),
      import: HashMap::new(),
      operations: HashMap::new(),
      tests: Vec::new(),
    }
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
    let mut manifest = Self::from_yaml(&contents, &Some(path.to_string_lossy().to_string()))?;
    manifest.source = Some(path.to_string_lossy().to_string());
    Ok(manifest)
  }

  /// Load struct from bytes by attempting to parse all the supported file formats.
  pub fn load_from_bytes(source: Option<String>, bytes: &[u8]) -> Result<ComponentConfiguration> {
    let contents = String::from_utf8_lossy(bytes);
    let mut manifest = Self::from_yaml(&contents, &source)?;
    manifest.source = source;
    Ok(manifest)
  }

  /// Load as YAML.
  pub fn from_yaml(src: &str, path: &Option<String>) -> Result<ComponentConfiguration> {
    debug!("Trying to parse manifest as yaml");
    let raw: serde_yaml::Value = from_yaml(src, path)?;
    debug!("Yaml parsed successfully");
    let raw_version = raw.get("format").ok_or(Error::NoFormat)?;
    let version = raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
    let manifest = match version {
      0 => Ok(from_yaml::<v0::HostManifest>(src, path)?.try_into()?),
      1 => Ok(from_yaml::<v1::V1ComponentConfiguration>(src, path)?.try_into()?),
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

  /// Get the `main:` implementation references if it exists.
  #[must_use]
  pub fn main(&self) -> Option<&ComponentImplementation> {
    self.main.as_ref()
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

  /// Return the list of tests defined in the manifest.
  #[must_use]
  pub fn tests(&self) -> &[TestCase] {
    &self.tests
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

  /// Get the name for this manifest.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn components(&self) -> &HashMap<String, BoundComponent> {
    &self.import
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn component(&self, namespace: &str) -> Option<&BoundComponent> {
    self.import.iter().find(|(k, _)| *k == namespace).map(|(_, v)| v)
  }

  /// Get a schematic by name
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&FlowOperation> {
    self.operations.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }

  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: v1::V1ComponentConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }
}

/// ComponentConfiguration builder.
#[derive(Default, Debug, Clone)]
#[must_use]
pub struct ComponentConfigurationBuilder {
  version: Option<String>,
  base: Option<ComponentConfiguration>,
  components: HashMap<String, BoundComponent>,
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
  pub fn add_collection(mut self, name: impl AsRef<str>, collection: BoundComponent) -> Self {
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
