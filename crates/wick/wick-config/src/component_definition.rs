use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde_json::Value;

use crate::v1;

/// A reference to an operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentOperationExpression {
  /// The operation ID.
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
      component: ComponentDefinition::Reference(ComponentReference { id: component }),
    })
  }
}

impl From<v1::ComponentOperationExpression> for ComponentOperationExpression {
  fn from(literal: v1::ComponentOperationExpression) -> Self {
    Self {
      operation: literal.operation,
      component: literal.component.into(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct BoundComponent {
  /// The namespace to reference the collection's components on.
  pub id: String,
  /// The kind/type of the collection.
  pub kind: ComponentDefinition,
}

impl BoundComponent {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: ComponentDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&Value> {
    match &self.kind {
      ComponentDefinition::Native(_) => None,
      ComponentDefinition::Wasm(v) => Some(&v.config),
      // ComponentDefinition::GrpcTar(v) => Some(&v.config),
      ComponentDefinition::GrpcUrl(v) => Some(&v.config),
      // ComponentDefinition::Mesh(v) => Some(&v.config),
      ComponentDefinition::Manifest(v) => Some(&v.config),
      ComponentDefinition::Reference(_) => panic!("Cannot get config for a reference"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
/// The kinds of collections that can operate in a flow.
pub enum ComponentDefinition {
  #[doc(hidden)]
  Native(NativeComponent),
  /// WebAssembly Collections.
  Wasm(WasmComponent),
  /// WebAssembly Collections.
  Reference(ComponentReference),
  // /// Archived, native binaries that Wick can fetch, extract, and run as a microservice.
  // GrpcTar(GrpcTarComponent),
  /// Separate microservices that Wick can connect to.
  GrpcUrl(GrpcUrlComponent),
  // /// Collections that exist over a connected mesh.
  // Mesh(MeshComponent),
  /// External manifests.
  Manifest(ManifestComponent),
}

#[derive(Debug, Clone, PartialEq)]
/// A reference to a component by id.
pub struct ComponentReference {
  id: String,
}

impl ComponentReference {
  /// Get the id of the referenced component.
  #[must_use]
  pub fn id(&self) -> &str {
    &self.id
  }
}

impl ComponentDefinition {
  /// Instantiate a new [CollectionKind].
  pub fn new(def: impl TryInto<ComponentDefinition>) -> Result<Self, crate::Error> {
    match def.try_into() {
      Ok(v) => Ok(v),
      Err(_e) => Err(crate::Error::Other("Could not load collection definition".to_owned())),
    }
  }

  /// Create a new [CollectionKind::Wasm] variant.
  pub fn wasm(reference: impl AsRef<str>, config: Option<Value>, permissions: Option<Permissions>) -> Self {
    Self::Wasm(WasmComponent {
      reference: reference.as_ref().to_owned(),
      config: config.unwrap_or_default(),
      permissions: permissions.unwrap_or_default(),
    })
  }

  /// Create a new [CollectionKind::GrpcUrl] variant.
  pub fn grpc_url(url: impl AsRef<str>, config: Option<Value>) -> Self {
    Self::GrpcUrl(GrpcUrlComponent {
      url: url.as_ref().to_owned(),
      config: config.unwrap_or_default(),
    })
  }

  /// Create a new [CollectionKind::Manifest] variant.
  pub fn manifest(reference: impl AsRef<str>, config: Option<Value>) -> Self {
    Self::Manifest(ManifestComponent {
      reference: reference.as_ref().to_owned(),
      config: config.unwrap_or_default(),
    })
  }

  /// Returns true if the definition is a reference to another component.
  #[must_use]
  pub fn is_reference(&self) -> bool {
    matches!(self, ComponentDefinition::Reference(_))
  }
}

/// A native collection compiled and built in to the runtime.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_copy_implementations)]
pub struct NativeComponent {}

/// A WebAssembly collection.
#[derive(Debug, Clone, PartialEq)]
pub struct WasmComponent {
  /// The OCI reference/local path of the collection.
  pub reference: String,
  /// The configuration for the collection
  pub config: Value,
  /// Permissions for this collection
  pub permissions: Permissions,
}

/// The permissions object for a collection
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Permissions {
  /// A map of directories (Note: TO -> FROM) to expose to the collection.
  #[serde(default)]
  pub dirs: std::collections::HashMap<String, String>,
}

/// A collection exposed as an external microservice.
#[derive(Debug, Clone, PartialEq)]
pub struct GrpcUrlComponent {
  /// The URL to connect to .
  pub url: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a collection.
  pub reference: String,
  /// The configuration for the collection
  pub config: Value,
}

impl TryFrom<&crate::v0::CollectionDefinition> for ComponentDefinition {
  type Error = crate::Error;
  fn try_from(def: &crate::v0::CollectionDefinition) -> Result<Self, Self::Error> {
    let kind = match def.kind {
      crate::v0::CollectionKind::Native => ComponentDefinition::Native(NativeComponent {}),
      crate::v0::CollectionKind::GrpcUrl => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::WaPC => ComponentDefinition::Wasm(WasmComponent {
        reference: def.reference.clone(),
        permissions: json_struct_to_permissions(def.data.get("wasi"))?,
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Network => ComponentDefinition::Manifest(ManifestComponent {
        reference: def.reference.clone(),
        config: def.data.clone(),
      }),
    };
    Ok(kind)
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

fn json_struct_to_permissions(json_perms: Option<&Value>) -> Result<Permissions, crate::Error> {
  let perms = if let Some(json_perms) = json_perms {
    serde_json::from_value(json_perms.clone()).map_err(crate::Error::Invalid)?
  } else {
    Permissions::default()
  };

  Ok(perms)
}

impl From<crate::v1::ComponentDefinition> for ComponentDefinition {
  fn from(def: crate::v1::ComponentDefinition) -> Self {
    match def {
      crate::v1::ComponentDefinition::WasmComponent(v) => ComponentDefinition::Wasm(WasmComponent {
        reference: v.reference,
        config: v.config,
        permissions: v.permissions.into(),
      }),
      crate::v1::ComponentDefinition::GrpcUrlComponent(v) => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: v.url,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ManifestComponent(v) => ComponentDefinition::Manifest(ManifestComponent {
        reference: v.reference,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ComponentReference(v) => {
        ComponentDefinition::Reference(ComponentReference { id: v.id })
      }
    }
  }
}

impl From<crate::v1::Permissions> for Permissions {
  fn from(def: crate::v1::Permissions) -> Self {
    Self { dirs: def.dirs }
  }
}
