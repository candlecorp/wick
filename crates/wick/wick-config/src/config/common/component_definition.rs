use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;

use crate::config;

/// A reference to an operation.
#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]

pub struct ComponentOperationExpression {
  /// The operation ID.
  #[asset(skip)]
  pub(crate) operation: String,
  /// The component referenced by identifier or anonymously.
  pub(crate) component: ComponentDefinition,
}

impl ComponentOperationExpression {
  /// Create a new [ComponentOperationExpression] with specified operation and component.
  pub fn new(operation: impl AsRef<str>, component: ComponentDefinition) -> Self {
    Self {
      operation: operation.as_ref().to_owned(),
      component,
    }
  }

  /// Returns the operation ID.
  #[must_use]
  pub fn operation(&self) -> &str {
    &self.operation
  }

  /// Returns the component definition.
  pub fn component(&self) -> &ComponentDefinition {
    &self.component
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
      operation,
      component: ComponentDefinition::Reference(config::components::ComponentReference { id: component }),
    })
  }
}

#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub enum HighLevelComponent {
  #[asset(skip)]
  Postgres(config::components::SqlComponentConfig),
}

#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
/// The kinds of collections that can operate in a flow.
#[must_use]
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
  pub fn is_reference(&self) -> bool {
    matches!(self, ComponentDefinition::Reference(_))
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
