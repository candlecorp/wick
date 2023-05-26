use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use wick_packet::OperationConfig;

use crate::config;

/// A reference to an operation.
#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]

pub struct ComponentOperationExpression {
  /// The operation ID.
  #[asset(skip)]
  pub(crate) name: String,
  /// The component referenced by identifier or anonymously.
  pub(crate) component: ComponentDefinition,
  /// Configuration to associate with this operation.
  #[asset(skip)]
  pub(crate) config: Option<OperationConfig>,
}

impl ComponentOperationExpression {
  /// Create a new [ComponentOperationExpression] with specified operation and component.
  pub fn new(operation: impl AsRef<str>, component: ComponentDefinition) -> Self {
    Self {
      name: operation.as_ref().to_owned(),
      component,
      config: None,
    }
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
      config: None,
    })
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub enum HighLevelComponent {
  #[asset(skip)]
  Sql(config::components::SqlComponentConfig),
  #[asset(skip)]
  HttpClient(config::components::HttpClientComponentConfig),
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
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

  /// Returns the component config, if it exists
  #[must_use]
  pub fn config(&self) -> Option<&OperationConfig> {
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
