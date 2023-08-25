use std::path::PathBuf;

use serde::de::DeserializeOwned;

use crate::{Error, Result};

/// A raw configuration, before it's been converted into a `WickConfiguration`.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum RawConfig {
  /// A v0 configuration.
  #[cfg(feature = "v0")]
  V0(Box<crate::v0::HostManifest>),

  /// A v1 configuration.
  #[cfg(feature = "v1")]
  V1(Box<crate::v1::WickConfig>),

  /// No configured features enabled.
  #[cfg(not(any(feature = "v0", feature = "v1")))]
  NoConfiguredFeatures,
}

/// Load a raw configuration from yaml source.
pub fn load_raw_config(src: &str, source: &Option<PathBuf>) -> Result<RawConfig> {
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
    #[cfg(feature = "v0")]
    0 => {
      let host_config = serde_yaml::from_str::<crate::v0::HostManifest>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      Ok(RawConfig::V0(Box::new(host_config)))
    }
    #[cfg(feature = "v1")]
    1 => {
      let base_config = serde_yaml::from_str::<crate::v1::WickConfig>(src)
        .map_err(|e| Error::YamlError(source.clone(), e.to_string(), e.location()))?;
      Ok(RawConfig::V1(Box::new(base_config)))
    }
    -1 => Err(Error::NoFormat(source.clone())),
    _ => Err(Error::VersionError(version.to_string())),
  }
}

#[cfg(feature = "config")]
pub(crate) fn resolve_configuration(
  src: &str,
  source: &Option<PathBuf>,
) -> Result<crate::config::UninitializedConfiguration> {
  let raw_config = load_raw_config(src, source)?;

  match raw_config {
    #[cfg(feature = "v0")]
    RawConfig::V0(config) => {
      let mut config = crate::config::WickConfiguration::Component((*config).try_into()?);
      if let Some(src) = source {
        config.set_source(src);
      }
      Ok(crate::config::UninitializedConfiguration::new(config))
    }
    #[cfg(feature = "v1")]
    RawConfig::V1(config) => {
      let mut config: crate::config::WickConfiguration = (*config).try_into()?;
      if let Some(src) = source {
        config.set_source(src);
      }
      Ok(crate::config::UninitializedConfiguration::new(config))
    }
    #[cfg(not(any(feature = "v0", feature = "v1")))]
    RawConfig::NoConfiguredFeatures => {
      panic!("No configured features enabled, please enable a version to convert from")
    }
  }
}

pub(crate) fn from_yaml<T>(src: &str, path: &Option<PathBuf>) -> Result<T>
where
  T: DeserializeOwned,
{
  let result =
    serde_yaml::from_str(src).map_err(|e| Error::YamlError(path.as_ref().cloned(), e.to_string(), e.location()))?;
  Ok(result)
}
