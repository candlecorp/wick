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

/// Utility macro for implementing `From` for a type.
#[allow(unused)]
macro_rules! impl_from_for {
  ($root:ident, $variant: ident, $type:ty) => {
    impl From<$type> for $root {
      fn from(value: $type) -> Self {
        Self::$variant(value)
      }
    }
    #[allow(unused_qualifications)]
    impl TryFrom<$root> for $type {
      type Error = crate::error::ManifestError;
      fn try_from(value: $root) -> std::result::Result<Self, Self::Error> {
        match value {
          $root::$variant(value) => Ok(value),
          _ => Err(Self::Error::VariantError(
            value.kind().to_string(),
            stringify!($type).to_owned(),
          )),
        }
      }
    }
  };
  ($root:ident, $variant: ident) => {
    crate::impl_from_for!($root, $variant, $variant);
  };
}
// has to be specified after the macro;
pub(crate) use impl_from_for;

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
