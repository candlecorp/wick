use std::net::Ipv4Addr;
use std::str::FromStr;

use crate::error::ManifestError;
use crate::Result;

#[cfg(feature = "config")]
mod config;
#[cfg(feature = "config")]
pub(crate) use config::*;

#[allow(unused)]
pub(crate) fn opt_str_to_ipv4addr(v: &Option<String>) -> Result<Option<Ipv4Addr>> {
  Ok(match v {
    Some(v) => Some(Ipv4Addr::from_str(v).map_err(|e| ManifestError::BadIpAddress(e.to_string()))?),
    None => None,
  })
}

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
