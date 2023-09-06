#![allow(missing_docs, deprecated)]
use std::collections::HashMap;
use std::path::Path;

// delete when we move away from the `property` crate.
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use wick_packet::{Entity, RuntimeConfig};

use super::template_config::Renderable;
use super::ImportBinding;
use crate::config::{self, ExecutionSettings, LiquidJsonConfig};
use crate::error::ManifestError;

/// A reference to an operation.
#[derive(
  Debug,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
  derive_builder::Builder,
)]
#[builder(setter(into))]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]

pub struct ComponentOperationExpression {
  /// The operation ID.
  #[asset(skip)]
  pub(crate) name: String,
  /// The component referenced by identifier or anonymously.
  pub(crate) component: ComponentDefinition,
  /// Configuration to associate with this operation.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) config: Option<LiquidJsonConfig>,
  /// Per-operation settings that override global execution settings.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) settings: Option<ExecutionSettings>,
}

impl ComponentOperationExpression {
  /// Create a new [ComponentOperationExpression] with specified operation and component.
  pub fn new_default<T: Into<String>>(operation: T, component: ComponentDefinition) -> Self {
    Self {
      name: operation.into(),
      component,
      config: Default::default(),
      settings: Default::default(),
    }
  }

  /// Create a new [ComponentOperationExpression] with specified operation and component.
  pub fn new<T: Into<String>>(
    operation: T,
    component: ComponentDefinition,
    config: Option<LiquidJsonConfig>,
    settings: Option<ExecutionSettings>,
  ) -> Self {
    Self {
      name: operation.into(),
      component,
      config,
      settings,
    }
  }

  pub fn maybe_import(&mut self, import_name: &str, bindings: &mut Vec<ImportBinding>) {
    if self.component.is_reference() {
      return;
    }

    tracing::Span::current().in_scope(|| {
      tracing::debug!("importing inline component as {}", import_name);
      let def = std::mem::replace(
        &mut self.component,
        ComponentDefinition::Reference(config::components::ComponentReference::new(import_name)),
      );
      bindings.push(ImportBinding::new(
        import_name,
        config::ImportDefinition::Component(def),
      ));
    });
  }

  /// Get the operation as an [Entity] if the component definition is a referencable instance.
  #[must_use]
  pub fn as_entity(&self) -> Option<Entity> {
    if let ComponentDefinition::Reference(r) = &self.component {
      Some(Entity::operation(&r.id, &self.name))
    } else {
      None
    }
  }
}

impl Renderable for ComponentOperationExpression {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    if let Some(config) = self.config.as_mut() {
      config.set_value(Some(config.render(source, root_config, None, env, None)?));
    }
    Ok(())
  }
}

impl std::str::FromStr for ComponentOperationExpression {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split("::");

    let operation = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();
    let component = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();

    Ok(Self {
      name: operation,
      component: ComponentDefinition::Reference(config::components::ComponentReference { id: component }),
      config: Default::default(),
      settings: Default::default(),
    })
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(config::AssetReference))]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
#[serde(rename_all = "kebab-case")]
pub enum HighLevelComponent {
  /// A SQL Component.
  #[asset(skip)]
  Sql(config::components::SqlComponentConfig),
  #[asset(skip)]
  /// An HTTP Client Component.
  HttpClient(config::components::HttpClientComponentConfig),
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(config::AssetReference))]

/// The kinds of collections that can operate in a flow.
#[must_use]
#[serde(rename_all = "kebab-case")]
pub enum ComponentDefinition {
  #[doc(hidden)]
  #[asset(skip)]
  Native(config::components::NativeComponent),
  /// WebAssembly Collections.
  #[deprecated(note = "Use ManifestComponent instead")]
  Wasm(config::components::WasmComponent),
  /// A component reference.
  #[asset(skip)]
  Reference(config::components::ComponentReference),
  /// Separate microservices that Wick can connect to.
  #[asset(skip)]
  GrpcUrl(config::components::GrpcUrlComponent),
  /// External manifests.
  Manifest(config::components::ManifestComponent),
  /// Postgres Component.
  #[asset(skip)]
  HighLevelComponent(HighLevelComponent),
}

impl ComponentDefinition {
  /// Returns true if the definition is a reference to another component.
  #[must_use]
  pub const fn is_reference(&self) -> bool {
    matches!(self, ComponentDefinition::Reference(_))
  }

  /// Returns the component config, if it exists
  #[must_use]
  pub const fn config(&self) -> Option<&LiquidJsonConfig> {
    match self {
      #[allow(deprecated)]
      ComponentDefinition::Wasm(c) => c.config.as_ref(),
      ComponentDefinition::GrpcUrl(c) => c.config.as_ref(),
      ComponentDefinition::Manifest(c) => c.config.as_ref(),
      ComponentDefinition::Native(_) => None,
      ComponentDefinition::Reference(_) => None,
      ComponentDefinition::HighLevelComponent(_) => None,
    }
  }

  /// Returns any components this configuration provides to the implementation.
  #[must_use]
  pub fn provide(&self) -> Option<&HashMap<String, String>> {
    match self {
      #[allow(deprecated)]
      ComponentDefinition::Wasm(c) => Some(c.provide()),
      ComponentDefinition::GrpcUrl(_) => None,
      ComponentDefinition::Manifest(c) => Some(c.provide()),
      ComponentDefinition::Native(_) => None,
      ComponentDefinition::Reference(_) => None,
      ComponentDefinition::HighLevelComponent(_) => None,
    }
  }

  /// Returns the component config, if it exists
  #[must_use]
  pub fn config_mut(&mut self) -> Option<&mut LiquidJsonConfig> {
    match self {
      #[allow(deprecated)]
      ComponentDefinition::Wasm(c) => c.config.as_mut(),
      ComponentDefinition::GrpcUrl(c) => c.config.as_mut(),
      ComponentDefinition::Manifest(c) => c.config.as_mut(),
      ComponentDefinition::Native(_) => None,
      ComponentDefinition::Reference(_) => None,
      ComponentDefinition::HighLevelComponent(_) => None,
    }
  }

  /// Returns the component config, if it exists
  pub fn set_config(&mut self, config: Option<RuntimeConfig>) {
    match self {
      #[allow(deprecated)]
      ComponentDefinition::Wasm(c) => c.config.as_mut().map(|c| c.set_value(config)),
      ComponentDefinition::GrpcUrl(c) => c.config.as_mut().map(|c| c.set_value(config)),
      ComponentDefinition::Manifest(c) => c.config.as_mut().map(|c| c.set_value(config)),
      ComponentDefinition::Native(_) => None,
      ComponentDefinition::Reference(_) => None,
      ComponentDefinition::HighLevelComponent(_) => None,
    };
  }
}

impl Renderable for ComponentDefinition {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    let val = if let Some(config) = self.config() {
      Some(config.render(source, root_config, None, env, None)?)
    } else {
      None
    };
    self.set_config(val);
    Ok(())
  }
}

#[derive(Default, Debug)]
struct StringPair(String, String);

impl<'de> serde::Deserialize<'de> for StringPair {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringPairVisitor;

    impl<'de> Visitor<'de> for StringPairVisitor {
      type Value = StringPair;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a String pair")
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
      where
        V: SeqAccess<'de>,
      {
        let s = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let n = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        // This is very important!
        while matches!(seq.next_element()?, Some(IgnoredAny)) {
          // Ignore rest
        }

        Ok(StringPair(s, n))
      }
    }

    deserializer.deserialize_seq(StringPairVisitor)
  }
}
