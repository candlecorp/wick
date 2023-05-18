use std::collections::HashMap;
use std::sync::Arc;

use asset_container::Asset;
use flow_component::BoxFuture;
use parking_lot::RwLock;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_interface_types::TypeDefinition;

use crate::common::{ImportBinding, ImportDefinition};
use crate::error::ManifestError;
use crate::types_config::TypesConfiguration;
use crate::WickConfiguration;

#[derive(Default, Clone, Debug, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
pub(crate) struct ImportCache {
  #[asset(skip)]
  pub(crate) cached_types: Arc<RwLock<HashMap<String, TypesConfiguration>>>,
}

impl ImportCache {
  /// Get a type from cache or fetch it as necessary.
  pub(crate) fn fetch_type(
    &self,
    name: impl AsRef<str>,
    asset: AssetReference,
    options: FetchOptions,
  ) -> BoxFuture<Result<TypesConfiguration, ManifestError>> {
    let name = name.as_ref().to_owned();
    let cache_item = self.cached_types.read().get(&name).cloned();
    let cache = self.cached_types.clone();

    Box::pin(async move {
      if let Some(c) = cache_item {
        Ok(c)
      } else {
        let bytes = asset.fetch(options).await.map_err(ManifestError::AssetContainer)?;
        let config = WickConfiguration::load_from_bytes(&bytes, &Some(asset.path().unwrap()))?;
        let type_config = config.try_types_config()?;
        cache.write().insert(name, type_config.clone());
        Ok(type_config)
      }
    })
  }
}

pub(crate) async fn setup_cache(
  cache: &ImportCache,
  imports: impl Iterator<Item = &ImportBinding> + Send,
  cached_types: &Arc<RwLock<Option<Vec<TypeDefinition>>>>,
  mut init: Vec<TypeDefinition>,
  options: FetchOptions,
) -> Result<(), ManifestError> {
  if cached_types.read().is_some() {
    return Ok(());
  }
  let mut types = Vec::new();
  for import in imports {
    let prefix = &import.id;
    if let ImportDefinition::Types(t) = &import.kind {
      let config = cache
        .fetch_type(&import.id, t.reference.clone(), options.clone())
        .await?;
      // if we didn't specify types to import, import everything.
      if t.types.is_empty() {
        for ty in config.into_types() {
          types.push(prefix_type(prefix, ty));
        }
      } else {
        // otherwise import a subset.
        for t in &t.types {
          if let Some(ty) = config.get_type(t) {
            types.push(prefix_type(prefix, ty.clone()));
          } else {
            return Err(ManifestError::TypeNotFound(t.clone()));
          }
        }
      }
    }
  }
  types.append(&mut init);
  *cached_types.write() = Some(types);
  Ok(())
}

fn prefix_type(prefix: &str, mut ty: TypeDefinition) -> TypeDefinition {
  match ty {
    TypeDefinition::Struct(ref mut ty) => {
      ty.name = format!("{}::{}", prefix, ty.name);
      ty.imported = true;
    }
    TypeDefinition::Enum(ref mut ty) => {
      ty.name = format!("{}::{}", prefix, ty.name);
      ty.imported = true;
    }
  }
  ty
}
