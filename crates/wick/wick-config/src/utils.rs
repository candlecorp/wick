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

pub(crate) trait OptMapInto<I> {
  fn map_into<R>(self) -> Option<R>
  where
    Self: Sized,
    I: Into<R>;
}

impl<I> OptMapInto<I> for Option<I> {
  fn map_into<R>(self) -> Option<R>
  where
    Self: Sized,
    I: Into<R>,
  {
    self.map(Into::into)
  }
}

pub(crate) trait OptMapTryInto<I> {
  fn map_try_into<R>(self) -> Result<Option<R>>
  where
    Self: Sized,
    I: TryInto<R, Error = ManifestError>;
}

impl<I> OptMapTryInto<I> for Option<I> {
  fn map_try_into<R>(self) -> Result<Option<R>>
  where
    Self: Sized,
    I: TryInto<R, Error = ManifestError>,
  {
    self.map(TryInto::try_into).transpose()
  }
}

// pub(crate) trait IterTryMapInto<I, ITER, O> {
//   fn try_map_into<R>(self) -> Result<O>
//   where
//     Self: Sized,
//     I: TryInto<R, Error = ManifestError>,
//     O: FromIterator<R>,
//     ITER: Iterator<Item = I>;
// }
// pub(crate) trait IterMapInto<I, ITER, O> {
//   fn map_into<R>(self) -> O
//   where
//     Self: Sized,
//     I: Into<R>,
//     O: FromIterator<R>,
//     ITER: Iterator<Item = I>;
// }

// impl<I, ITER, O> IterTryMapInto<I, ITER, O> for ITER {
//   fn try_map_into<R>(self) -> Result<O>
//   where
//     Self: Sized,
//     I: TryInto<R, Error = ManifestError>,
//     O: FromIterator<R>,
//     ITER: Iterator<Item = I>,
//   {
//     self.into_iter().map(TryInto::try_into).collect::<Result<O>>()
//   }
// }

// impl<I, ITER, O> IterMapInto<I, ITER, O> for ITER {
//   fn map_into<R>(self) -> O
//   where
//     Self: Sized,
//     I: Into<R>,
//     O: FromIterator<R>,
//     ITER: Iterator<Item = I>,
//   {
//     self.into_iter().map(Into::into).collect::<O>()
//   }
// }
