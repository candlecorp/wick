use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::ParseError as Error;

#[derive(Debug, Clone, PartialEq)]
/// The entity being referenced across systems or services.
#[must_use]
pub enum Entity {
  /// An invalid entity. Used only for situations where a default is necessary.
  Invalid,
  /// A "test" entity. Used as the originating entity for tests.
  Test(String),
  /// A server or host entity (i.e. for requests).
  Server(String),
  /// An operation or anything that can be invoked like an operation.
  Operation(String, String),
  /// A component that hosts a collection of operations.
  Component(String),
}

impl Serialize for Entity {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.collect_str(&self)
  }
}

impl<'de> Deserialize<'de> for Entity {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(serde::de::Error::custom)
  }
}

impl Default for Entity {
  fn default() -> Self {
    Self::Test("default".to_owned())
  }
}

impl Display for Entity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url())
  }
}

pub(crate) const URL_SCHEME: &str = "wick";

impl FromStr for Entity {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use url::Url;
    let url = Url::parse(s).map_err(Error::Parse)?;
    if url.scheme() != URL_SCHEME {
      return Err(Error::Scheme(url.scheme().to_owned()));
    }
    let host = url.host_str().ok_or(Error::Authority)?;
    if let Some((id, kind)) = host.split_once('.') {
      if kind == "host" {
        return Ok(Entity::server(id));
      }
      return Err(Error::InvalidAuthority(host.to_owned()));
    }

    match host {
      "__test__" => {
        let (_, msg) = url
          .query_pairs()
          .find(|(k, _v)| k == "msg")
          .unwrap_or(("".into(), "".into()));
        Ok(Entity::test(msg))
      }
      "__invalid__" => Ok(Entity::Invalid),
      _ => {
        if let Some(mut segments) = url.path_segments() {
          if let Some(name) = segments.next() {
            if !name.is_empty() {
              return Ok(Entity::operation(host, name));
            }
          }
        }
        Ok(Entity::component(host))
      }
    }
  }
}

impl TryFrom<&str> for Entity {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Self::from_str(value)
  }
}

impl Entity {
  /// Namespace for components local to a collection.
  pub const LOCAL: &'static str = "__local__";

  /// Constructor for [Entity::Component].
  pub fn operation<T: AsRef<str>, U: AsRef<str>>(ns: T, name: U) -> Self {
    Self::Operation(ns.as_ref().to_owned(), name.as_ref().to_owned())
  }

  /// Constructor for [Entity::Component] on the local namespace, used when
  /// the namespace is irrelevant. Caution: this is not portable.
  pub fn local<T: AsRef<str>>(name: T) -> Self {
    Self::Operation(Self::LOCAL.to_owned(), name.as_ref().to_owned())
  }

  /// Constructor for an [Entity::Test].
  pub fn test<T: AsRef<str>>(msg: T) -> Self {
    Self::Test(msg.as_ref().to_owned())
  }

  /// Constructor for an [Entity::Component].
  pub fn component<T: AsRef<str>>(id: T) -> Self {
    Self::Component(id.as_ref().to_owned())
  }

  /// Constructor for [Entity::Server].
  pub fn server<T: AsRef<str>>(id: T) -> Self {
    Self::Server(id.as_ref().to_owned())
  }

  /// The URL of the entity.
  #[must_use]
  pub fn url(&self) -> String {
    match self {
      Entity::Test(msg) => format!("{}://__test__/?msg={}", URL_SCHEME, msg),
      Entity::Operation(ns, id) => format!("{}://{}/{}", URL_SCHEME, ns, id),
      Entity::Component(name) => format!("{}://{}/", URL_SCHEME, name),
      Entity::Server(id) => format!("{}://{}.host/", URL_SCHEME, id),
      Entity::Invalid => format!("{}://__invalid__/", URL_SCHEME),
    }
  }

  /// The name of the entity.
  #[must_use]
  pub fn operation_id(&self) -> &str {
    match self {
      Entity::Test(_) => "",
      Entity::Operation(_, id) => id,
      Entity::Component(_) => "",
      Entity::Server(_) => "",
      Entity::Invalid => "",
    }
  }

  pub fn set_operation(&mut self, id: impl AsRef<str>) {
    match self {
      Entity::Test(_) => {}
      Entity::Operation(_, op_id) => *op_id = id.as_ref().to_owned(),
      Entity::Component(comp_id) => *self = Entity::operation(comp_id, id.as_ref()),
      Entity::Server(_) => {}
      Entity::Invalid => {}
    }
  }

  /// The id of the component entity.
  #[must_use]
  pub fn component_id(&self) -> &str {
    match self {
      Entity::Test(_) => "test",
      Entity::Operation(id, _) => id,
      Entity::Component(name) => name,
      Entity::Server(id) => id,
      Entity::Invalid => "<invalid>",
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn test() -> Result<(), Error> {
    let entity = Entity::from_str("wick://namespace/comp_name")?;
    assert_eq!(entity, Entity::operation("namespace", "comp_name"));

    let entity = Entity::from_str("wick://some_ns/")?;
    assert_eq!(entity, Entity::component("some_ns"));

    let entity = Entity::from_str("wick://client_id.host/")?;
    assert_eq!(entity, Entity::server("client_id"));

    let entity = Entity::from_str("wick://__test__/?msg=Hello")?;
    assert_eq!(entity, Entity::test("Hello"));

    Ok(())
  }
}
