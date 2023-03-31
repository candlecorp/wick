use std::net::Ipv4Addr;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use url::Url;

use crate::error::ManifestError;
use crate::{Error, Result};

pub(crate) fn opt_str_to_ipv4addr(v: &Option<String>) -> Result<Option<Ipv4Addr>> {
  Ok(match v {
    Some(v) => Some(Ipv4Addr::from_str(v).map_err(|e| ManifestError::BadIpAddress(e.to_string()))?),
    None => None,
  })
}

pub(crate) fn from_yaml<T>(src: &str, path: &Option<Url>) -> Result<T>
where
  T: DeserializeOwned,
{
  let result =
    serde_yaml::from_str(src).map_err(|e| Error::YamlError(path.as_ref().map(|v| v.as_ref().into()), e.to_string()))?;
  Ok(result)
}

pub(crate) fn from_bytes<T>(src: &[u8], path: &Option<Url>) -> Result<T>
where
  T: DeserializeOwned,
{
  let result = serde_yaml::from_slice(src)
    .map_err(|e| Error::YamlError(path.as_ref().map(|v| v.as_ref().into()), e.to_string()))?;
  Ok(result)
}

pub fn path_to_url(path: &std::path::Path, base: Option<Url>) -> Result<Url> {
  let pathstr = path.to_string_lossy().to_string();
  str_to_url(&pathstr, base)
}

pub fn str_to_url(path: &str, base: Option<Url>) -> Result<Url> {
  let url = match base {
    Some(full_url) => {
      if !full_url.path().ends_with('/') {
        let mut url = full_url.clone();
        url.set_path(&format!("{}/", full_url.path()));
        url.join(path)?
      } else {
        full_url.join(path)?
      }
    }
    None => match Url::from_str(path) {
      Ok(url) => url,
      Err(e) => match e {
        url::ParseError::RelativeUrlWithoutBase => {
          let mut cwd = std::env::current_dir().unwrap();
          cwd.push(path);
          Url::from_file_path(cwd).map_err(|_| Error::BadUrl(path.to_owned()))?
        }
        e => return Err(e.into()),
      },
    },
  };
  Ok(url)
}
