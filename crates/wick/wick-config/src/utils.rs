use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::de::DeserializeOwned;

use crate::error::ManifestError;
use crate::{Error, Result};

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
  let result = serde_yaml::from_str(src).map_err(|e| Error::YamlError(path.as_ref().cloned(), e.to_string()))?;
  Ok(result)
}

pub(crate) fn from_bytes<T>(src: &[u8], path: &Option<PathBuf>) -> Result<T>
where
  T: DeserializeOwned,
{
  let result = serde_yaml::from_slice(src).map_err(|e| Error::YamlError(path.as_ref().cloned(), e.to_string()))?;
  Ok(result)
}

pub(crate) type RwOption<T> = Arc<RwLock<Option<T>>>;
