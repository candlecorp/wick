use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::ManifestError;
use crate::Result;

pub(crate) fn opt_str_to_pathbuf(v: &Option<String>) -> Result<Option<PathBuf>> {
  Ok(match v {
    Some(v) => Some(PathBuf::from_str(v).map_err(|e| ManifestError::BadPath(e.to_string()))?),
    None => None,
  })
}

pub(crate) fn opt_str_to_ipv4addr(v: &Option<String>) -> Result<Option<Ipv4Addr>> {
  Ok(match v {
    Some(v) => Some(Ipv4Addr::from_str(v).map_err(|e| ManifestError::BadIpAddress(e.to_string()))?),
    None => None,
  })
}
