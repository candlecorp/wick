mod composite;
mod wasm;
use std::collections::HashMap;

pub use composite::CompositeComponentConfiguration;
pub use wasm::{OperationSignature, WasmComponentConfiguration};

use crate::host_definition::HostConfig;
use crate::{v1, Error, Result, TestCase};

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum ComponentKind {
  Wasm,
  Composite,
}

impl std::fmt::Display for ComponentKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ComponentKind::Wasm => write!(f, "wick/component/wasm"),
      ComponentKind::Composite => write!(f, "wick/component/composite"),
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub enum ComponentImplementation {
  Wasm(WasmComponentConfiguration),
  Composite(CompositeComponentConfiguration),
}

impl ComponentImplementation {
  pub fn kind(&self) -> ComponentKind {
    match self {
      ComponentImplementation::Wasm(_) => ComponentKind::Wasm,
      ComponentImplementation::Composite(_) => ComponentKind::Composite,
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct ComponentConfiguration {
  pub name: Option<String>,
  pub(crate) source: Option<String>,
  pub(crate) format: u32,
  pub(crate) version: String,
  pub(crate) host: HostConfig,
  pub(crate) labels: HashMap<String, String>,
  pub(crate) tests: Vec<TestCase>,
  pub(crate) component: ComponentImplementation,
}

impl ComponentConfiguration {
  // /// Load struct from file by trying all the supported file formats.
  // pub fn load_from_file(path: impl AsRef<Path>) -> Result<ComponentConfiguration> {
  //   let path = path.as_ref();
  //   if !path.exists() {
  //     return Err(Error::FileNotFound(path.to_string_lossy().into()));
  //   }
  //   debug!("Reading manifest from {}", path.to_string_lossy());
  //   let contents = read_to_string(path)?;
  //   let mut manifest = Self::from_yaml(&contents, &Some(path.to_string_lossy().to_string()))?;
  //   manifest.source = Some(path.to_string_lossy().to_string());
  //   Ok(manifest)
  // }

  // /// Load struct from bytes by attempting to parse all the supported file formats.
  // pub fn load_from_bytes(source: Option<String>, bytes: &[u8]) -> Result<ComponentConfiguration> {
  //   let contents = String::from_utf8_lossy(bytes);
  //   let mut manifest = Self::from_yaml(&contents, &source)?;
  //   manifest.source = source;
  //   Ok(manifest)
  // }

  // /// Load as YAML.
  // pub fn from_yaml(src: &str, path: &Option<String>) -> Result<ComponentConfiguration> {
  //   debug!("Trying to parse manifest as yaml");
  //   let raw: serde_yaml::Value = from_yaml(src, path)?;
  //   debug!("Yaml parsed successfully");
  //   let raw_version = raw.get("format").ok_or(Error::NoFormat)?;
  //   let version = raw_version
  //     .as_i64()
  //     .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) });
  //   let manifest = match version {
  //     0 => Ok(from_yaml::<v0::HostManifest>(src, path)?.try_into()?),
  //     1 => Ok(from_yaml::<v1::ComponentConfiguration>(src, path)?.try_into()?),
  //     -1 => Err(Error::NoFormat),
  //     _ => Err(Error::VersionError(version.to_string())),
  //   };

  //   debug!("Manifest: {:?}", manifest);
  //   manifest
  // }

  pub fn try_composite(&self) -> Result<&CompositeComponentConfiguration> {
    match &self.component {
      ComponentImplementation::Composite(c) => Ok(c),
      _ => Err(Error::UnexpectedComponentType(
        ComponentKind::Composite,
        self.component.kind(),
      )),
    }
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host(&self) -> &HostConfig {
    &self.host
  }

  /// Determine if the configuration allows for fetching artifacts with the :latest tag.
  pub fn host_mut(&mut self) -> &mut HostConfig {
    &mut self.host
  }

  /// Get the configuration related to the specific [ComponentKind].
  pub fn component(&self) -> &ComponentImplementation {
    &self.component
  }

  /// Get the kind of this component implementation.
  pub fn kind(&self) -> ComponentKind {
    self.component.kind()
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
  /// Get the name for this manifest.
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  #[must_use]
  /// Get the name for this manifest.
  pub fn labels(&self) -> &HashMap<String, String> {
    &self.labels
  }

  pub fn into_v1_yaml(self) -> Result<String> {
    let v1_manifest: v1::ComponentConfiguration = self.try_into()?;
    Ok(serde_yaml::to_string(&v1_manifest).unwrap())
  }
}

/// ComponentConfiguration builder.
#[derive(Debug, Clone)]
#[must_use]
pub struct ComponentConfigurationBuilder {
  version: Option<String>,
  name: Option<String>,
  source: Option<String>,
  format: u32,
  host: HostConfig,
  labels: HashMap<String, String>,
  tests: Vec<TestCase>,
  component: ComponentImplementation,
}

impl ComponentConfigurationBuilder {
  /// Create a new [ComponentConfigurationBuilder].
  pub fn new(component: ComponentImplementation) -> Self {
    Self {
      version: Default::default(),
      name: Default::default(),
      source: Default::default(),
      format: 1,
      host: Default::default(),
      labels: Default::default(),
      tests: Default::default(),
      component,
    }
  }

  // /// Create a builder with an existing manifest as a base.
  // pub fn with_base(definition: ComponentConfiguration) -> Self {
  //   Self {
  //     base: Some(definition),
  //     ..Default::default()
  //   }
  // }

  /// Set the version of the component.
  pub fn version(mut self, version: impl AsRef<str>) -> Self {
    self.version = Some(version.as_ref().to_owned());
    self
  }

  /// Consume the [ComponentConfigurationBuilder] and return a [ComponentConfiguration].
  pub fn build(self) -> ComponentConfiguration {
    // if let Some(mut def) = self.base {
    //   def.component = self.kind;

    //   def
    // } else {
    ComponentConfiguration {
      component: self.component,
      name: self.name,
      source: self.source,
      format: self.format,
      version: self.version.unwrap_or_else(|| "0.0.0".to_owned()),
      host: self.host,
      labels: self.labels,
      tests: self.tests,
    }
    // }
  }
}
