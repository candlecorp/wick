use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde_json::Value;

#[derive(Debug, Clone)]
/// A definition of a Wasmflow Collection with its namespace, how to retrieve or access it and its configuration.
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

impl TryFrom<crate::v1::EntrypointDefinition> for EntrypointDefinition {
  type Error = crate::Error;
  fn try_from(def: crate::v1::EntrypointDefinition) -> Result<Self, Self::Error> {
    Ok(EntrypointDefinition {
      reference: def.reference,
      config: def.config,
      permissions: def.permissions.into(),
      component: def.component,
    })
  }
}

#[derive(Debug, Clone)]
/// A definition of a Wasmflow Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct CollectionDefinition {
  /// The namespace to reference the collection's components on.
  pub namespace: String,
  /// The kind/type of the collection.
  pub kind: CollectionKind,
}

impl CollectionDefinition {
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
  Wasm(WasmCollection),
  /// Archived, native binaries that Wasmflow can fetch, extract, and run as a microservice.
  GrpcTar(GrpcTarCollection),
  /// Separate microservices that Wasmflow can connect to.
  GrpcUrl(GrpcUrlCollection),
  /// Collections that exist over a connected mesh.
  Mesh(MeshCollection),
  /// External manifests.
  Manifest(ManifestCollection),
}

/// A native collection compiled and built in to the runtime.
#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct NativeCollection {}

/// A WebAssembly collection.
#[derive(Debug, Clone)]
pub struct WasmCollection {
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
pub struct GrpcTarCollection {
  /// The OCI reference/local path of the collection.
  pub reference: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A collection exposed as an external microservice.
#[derive(Debug, Clone)]
pub struct GrpcUrlCollection {
  /// The URL to connect to .
  pub url: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A collection exposed over the connected mesh.
#[derive(Debug, Clone)]
pub struct MeshCollection {
  /// The ID of the collection on the mesh.
  pub id: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A separate Wasmflow manifest to use as a collection.
#[derive(Debug, Clone)]
pub struct ManifestCollection {
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
      crate::v0::CollectionKind::GrpcUrl => CollectionKind::GrpcUrl(GrpcUrlCollection {
        url: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::WaPC => CollectionKind::Wasm(WasmCollection {
        reference: def.reference.clone(),
        permissions: json_struct_to_permissions(def.data.get("wasi"))?,
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Mesh => CollectionKind::Mesh(MeshCollection {
        id: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Network => CollectionKind::Manifest(ManifestCollection {
        reference: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::GrpcTar => CollectionKind::GrpcTar(GrpcTarCollection {
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
        while let Some(IgnoredAny) = seq.next_element()? {
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

impl From<(String, crate::v1::CollectionDefinition)> for CollectionDefinition {
  fn from(def: (String, crate::v1::CollectionDefinition)) -> Self {
    CollectionDefinition {
      namespace: def.0,
      kind: match def.1 {
        crate::v1::CollectionDefinition::WasmCollection(v) => CollectionKind::Wasm(WasmCollection {
          reference: v.reference,
          config: v.config,
          permissions: v.permissions.into(),
        }),
        crate::v1::CollectionDefinition::GrpcUrlCollection(v) => CollectionKind::GrpcUrl(GrpcUrlCollection {
          url: v.url,
          config: v.config,
        }),
        crate::v1::CollectionDefinition::GrpcTarCollection(v) => CollectionKind::GrpcTar(GrpcTarCollection {
          reference: v.reference,
          config: v.config,
        }),
        crate::v1::CollectionDefinition::MeshCollection(v) => CollectionKind::Mesh(MeshCollection {
          id: v.id,
          config: v.config,
        }),
        crate::v1::CollectionDefinition::ManifestCollection(v) => CollectionKind::Manifest(ManifestCollection {
          reference: v.reference,
          config: v.config,
        }),
      },
    }
  }
}

impl From<crate::v1::Permissions> for Permissions {
  fn from(def: crate::v1::Permissions) -> Self {
    Self { dirs: def.dirs }
  }
}
