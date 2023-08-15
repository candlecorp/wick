use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use asset_container::{Asset, AssetFlags, AssetManager};
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use wick_asset_reference::{AssetReference, FetchOptions};
use wick_packet::RuntimeConfig;

use crate::config::template_config::Renderable;
use crate::config::{
  ImportBinding,
  ImportDefinition,
  OwnedConfigurationItem,
  ResourceBinding,
  UninitializedConfiguration,
};
use crate::error::ManifestError;
use crate::{v0, v1, Error, Resolver, Result, WickConfiguration};

pub(crate) fn opt_str_to_ipv4addr(v: &Option<String>) -> Result<Option<Ipv4Addr>> {
  Ok(match v {
    Some(v) => Some(Ipv4Addr::from_str(v).map_err(|e| ManifestError::BadIpAddress(e.to_string()))?),
    None => None,
  })
}

pub(crate) fn from_yaml<T>(src: &str, path: &Option<PathBuf>) -> Result<T>
where
  T: DeserializeOwned,
{
  let result =
    serde_yaml::from_str(src).map_err(|e| Error::YamlError(path.as_ref().cloned(), e.to_string(), e.location()))?;
  Ok(result)
}

pub(crate) type RwOption<T> = Arc<RwLock<Option<T>>>;

pub(crate) trait VecTryMapInto<I> {
  fn try_map_into<R>(self) -> Result<Vec<R>>
  where
    Self: Sized,
    I: TryInto<R, Error = ManifestError>;
}

impl<I> VecTryMapInto<I> for Vec<I> {
  fn try_map_into<R>(self) -> Result<Vec<R>>
  where
    Self: Sized,
    I: TryInto<R, Error = ManifestError>,
  {
    self.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>>>()
  }
}

pub(crate) trait VecMapInto<I> {
  fn map_into<R>(self) -> Vec<R>
  where
    Self: Sized,
    I: Into<R>;
}

impl<I> VecMapInto<I> for Vec<I> {
  fn map_into<R>(self) -> Vec<R>
  where
    Self: Sized,
    I: Into<R>,
  {
    self.into_iter().map(Into::into).collect::<Vec<_>>()
  }
}

pub(super) async fn fetch_all(
  asset_manager: &(dyn AssetManager<Asset = AssetReference> + Send + Sync),
  options: FetchOptions,
) -> Result<()> {
  for asset in asset_manager.assets().iter() {
    if asset.get_asset_flags() == AssetFlags::Lazy {
      continue;
    }
    asset.fetch(options.clone()).await?;
  }
  Ok(())
}

pub(crate) fn resolve_configuration(src: &str, source: &Option<PathBuf>) -> Result<UninitializedConfiguration> {
  let raw: serde_yaml::Value = from_yaml(src, source)?;

  let raw_version = raw.get("format");
  let raw_kind = raw.get("kind");
  let version = if raw_kind.is_some() {
    1
  } else {
    let raw_version = raw_version.ok_or(Error::NoFormat(source.clone()))?;
    raw_version
      .as_i64()
      .unwrap_or_else(|| -> i64 { raw_version.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1) })
  };
  // re-parse the yaml into the correct version from string again for location info.
  match version {
    0 => {
      let host_config = serde_yaml::from_str::<v0::HostManifest>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      let mut config = WickConfiguration::Component(host_config.try_into()?);
      if let Some(src) = source {
        config.set_source(src);
      }
      Ok(UninitializedConfiguration::new(config))
    }
    1 => {
      let base_config = serde_yaml::from_str::<v1::WickConfig>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      let mut config: WickConfiguration = base_config.try_into()?;
      if let Some(src) = source {
        config.set_source(src);
      }
      Ok(UninitializedConfiguration::new(config))
    }
    -1 => Err(Error::NoFormat(source.clone())),
    _ => Err(Error::VersionError(version.to_string())),
  }
}

pub(crate) fn make_resolver(
  imports: Vec<ImportBinding>,
  resources: Vec<ResourceBinding>,
  runtime_config: Option<RuntimeConfig>,
  env: Option<HashMap<String, String>>,
) -> Box<Resolver> {
  Box::new(move |name| resolve(name, &imports, &resources, runtime_config.as_ref(), env.as_ref()))
}

pub(crate) fn resolve(
  name: &str,
  imports: &[ImportBinding],
  resources: &[ResourceBinding],
  runtime_config: Option<&RuntimeConfig>,
  env: Option<&HashMap<String, String>>,
) -> Result<OwnedConfigurationItem> {
  if let Some(import) = imports.iter().find(|i| i.id == name) {
    match &import.kind {
      ImportDefinition::Component(component) => {
        let mut component = component.clone();
        return match component.render_config(runtime_config, env) {
          Ok(_) => Ok(OwnedConfigurationItem::Component(component)),
          Err(e) => Err(e),
        };
      }
      ImportDefinition::Types(_) => todo!(),
    }
  }
  if let Some(resource) = resources.iter().find(|i| i.id == name) {
    let mut resource = resource.kind.clone();
    return match resource.render_config(runtime_config, env) {
      Ok(_) => Ok(OwnedConfigurationItem::Resource(resource)),
      Err(e) => Err(e),
    };
  }
  Err(Error::IdNotFound(name.to_owned()))
}
