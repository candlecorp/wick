use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde_json::Value;

#[derive(Debug, Clone)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct EntrypointDefinition {
  /// The reference/location of the collection.
  pub reference: String,
  /// Data or configuration to pass to the collection initialization.
  pub config: Value,
  /// Permissions for this collection
  pub permissions: Permissions,
  /// The component to use as the entrypoint
  pub component: String,
}

impl TryFrom<crate::v0::EntrypointDefinition> for EntrypointDefinition {
  type Error = crate::Error;
  fn try_from(def: crate::v0::EntrypointDefinition) -> Result<Self, Self::Error> {
    Ok(EntrypointDefinition {
      permissions: json_struct_to_permissions(def.data.get("wasi")).unwrap_or_default(),
      reference: def.reference,
      config: def.data,
      component: def.component,
    })
  }
}

#[derive(Debug, Clone)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct CollectionDefinition {
  /// The namespace to reference the collection's components on.
  pub namespace: String,
  /// The kind/type of the collection.
  pub kind: CollectionKind,
}

impl CollectionDefinition {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: CollectionKind) -> Self {
    Self {
      namespace: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&Value> {
    match &self.kind {
      CollectionKind::Native(_) => None,
      CollectionKind::Wasm(v) => Some(&v.config),
      CollectionKind::GrpcTar(v) => Some(&v.config),
      CollectionKind::GrpcUrl(v) => Some(&v.config),
      CollectionKind::Mesh(v) => Some(&v.config),
      CollectionKind::Manifest(v) => Some(&v.config),
    }
  }
}

#[derive(Debug, Clone)]
/// The kinds of collections that can operate in a flow.
pub enum CollectionKind {
  #[doc(hidden)]
  Native(NativeCollection),
  /// WebAssembly Collections.
  Wasm(WasmComponent),
  /// Archived, native binaries that Wick can fetch, extract, and run as a microservice.
  GrpcTar(GrpcTarComponent),
  /// Separate microservices that Wick can connect to.
  GrpcUrl(GrpcUrlComponent),
  /// Collections that exist over a connected mesh.
  Mesh(MeshComponent),
  /// External manifests.
  Manifest(ManifestComponent),
}

impl CollectionKind {
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

  /// Create a new [CollectionKind::GrpcTar] variant.
  pub fn grpc_tar(reference: impl AsRef<str>, config: Option<Value>) -> Self {
    Self::GrpcTar(GrpcTarComponent {
      reference: reference.as_ref().to_owned(),
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

  /// Create a new [CollectionKind::Mesh] variant.
  pub fn mesh(id: impl AsRef<str>, config: Option<Value>) -> Self {
    Self::Mesh(MeshComponent {
      config: config.unwrap_or_default(),
      id: id.as_ref().to_owned(),
    })
  }
}

/// A native collection compiled and built in to the runtime.
#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct NativeCollection {}

/// A WebAssembly collection.
#[derive(Debug, Clone)]
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

/// A native binary that can be run as a GRPC microservice.
#[derive(Debug, Clone)]
pub struct GrpcTarComponent {
  /// The OCI reference/local path of the collection.
  pub reference: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A collection exposed as an external microservice.
#[derive(Debug, Clone)]
pub struct GrpcUrlComponent {
  /// The URL to connect to .
  pub url: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A collection exposed over the connected mesh.
#[derive(Debug, Clone)]
pub struct MeshComponent {
  /// The ID of the collection on the mesh.
  pub id: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone)]
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a collection.
  pub reference: String,
  /// The configuration for the collection
  pub config: Value,
}

impl TryFrom<&crate::v0::CollectionDefinition> for CollectionDefinition {
  type Error = crate::Error;
  fn try_from(def: &crate::v0::CollectionDefinition) -> Result<Self, Self::Error> {
    let kind = match def.kind {
      crate::v0::CollectionKind::Native => CollectionKind::Native(NativeCollection {}),
      crate::v0::CollectionKind::GrpcUrl => CollectionKind::GrpcUrl(GrpcUrlComponent {
        url: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::WaPC => CollectionKind::Wasm(WasmComponent {
        reference: def.reference.clone(),
        permissions: json_struct_to_permissions(def.data.get("wasi"))?,
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Mesh => CollectionKind::Mesh(MeshComponent {
        id: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Network => CollectionKind::Manifest(ManifestComponent {
        reference: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::GrpcTar => CollectionKind::GrpcTar(GrpcTarComponent {
        reference: def.reference.clone(),
        config: def.data.clone(),
      }),
    };
    Ok(CollectionDefinition {
      namespace: def.namespace.clone(),
      kind,
    })
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

impl From<(String, crate::v1::ComponentDefinition)> for CollectionDefinition {
  fn from(def: (String, crate::v1::ComponentDefinition)) -> Self {
    CollectionDefinition {
      namespace: def.0,
      kind: def.1.into(),
    }
  }
}

impl From<crate::v1::ComponentDefinition> for CollectionKind {
  fn from(def: crate::v1::ComponentDefinition) -> Self {
    match def {
      crate::v1::ComponentDefinition::WasmComponent(v) => CollectionKind::Wasm(WasmComponent {
        reference: v.reference,
        config: v.config,
        permissions: v.permissions.into(),
      }),
      crate::v1::ComponentDefinition::GrpcUrlComponent(v) => CollectionKind::GrpcUrl(GrpcUrlComponent {
        url: v.url,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::GrpcTarComponent(v) => CollectionKind::GrpcTar(GrpcTarComponent {
        reference: v.reference,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::MeshComponent(v) => CollectionKind::Mesh(MeshComponent {
        id: v.id,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ManifestComponent(v) => CollectionKind::Manifest(ManifestComponent {
        reference: v.reference,
        config: v.config,
      }),
    }
  }
}

impl From<crate::v1::Permissions> for Permissions {
  fn from(def: crate::v1::Permissions) -> Self {
    Self { dirs: def.dirs }
  }
}
