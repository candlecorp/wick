use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::EntityError as Error;

#[derive(Debug, Clone, PartialEq)]
/// The entity being referenced across systems or services.
#[must_use]
pub enum Entity {
  /// A [SystemEntity] with the name "invalid". Used only for situations where a default is more useful than an error.
  Invalid,
  /// The [SystemEntity] is used when communicating to or from the internals of another component. Used mostly by library developers.
  System(SystemEntity),
  /// A [SystemEntity] with the name "test". Used as the originating entity for tests.
  Test(String),
  /// A client entity used for requests.
  Client(String),
  /// A Host entity used for entities that serve responses to requests.
  Host(String),
  /// A schematic.
  Schematic(String),
  /// A component or anything that can be invoked like a component.
  Component(String, String),
  /// A provider (an entity that hosts a collection of components).
  Provider(String),
  /// A reference to an instance of an entity.
  Reference(String),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A struct to hold additional data for [SystemEntity]s.
pub struct SystemEntity {
  /// The name of the [SystemEntity].
  pub name: String,
  /// A freefrom string.
  pub value: String,
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

pub(crate) const URL_SCHEME: &str = "ofp";

impl FromStr for Entity {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use url::Url;
    let url = Url::parse(s).map_err(|e| Error::ParseError(e.to_string()))?;
    ensure!(
      url.scheme() == URL_SCHEME,
      Error::ParseError(format!("Invalid scheme: {}", url.scheme()))
    );
    let host = url
      .host_str()
      .ok_or_else(|| Error::ParseError("No authority supplied".to_owned()))?;
    let (id, kind) = host
      .split_once('.')
      .ok_or_else(|| Error::ParseError(format!("Invalid authority format '{}', no dot.", host)))?;
    match kind {
      "sys" => {
        let (_, msg) = url
          .query_pairs()
          .find(|(k, _v)| k == "msg")
          .unwrap_or(("".into(), "".into()));

        if id == "test" {
          Ok(Entity::test(msg))
        } else {
          Ok(Entity::system(id, msg))
        }
      }
      "ref" => Ok(Entity::reference(id)),
      "schem" => Ok(Entity::schematic(id)),
      "prov" => {
        if let Some(mut segments) = url.path_segments() {
          if let Some(name) = segments.next() {
            if !name.is_empty() {
              return Ok(Entity::component(id, name));
            }
          }
        }

        Ok(Entity::provider(id))
      }
      "client" => Ok(Entity::client(id)),
      "host" => Ok(Entity::host(id)),
      _ => Err(Error::ParseError(format!("Invalid authority kind: {}", kind))),
    }
  }
}
impl Entity {
  /// Namespace for components local to a provider.
  pub const LOCAL: &'static str = "__local__";

  /// Constructor for [Entity::Component].
  pub fn component<T: AsRef<str>, U: AsRef<str>>(ns: T, name: U) -> Self {
    Self::Component(ns.as_ref().to_owned(), name.as_ref().to_owned())
  }

  /// Constructor for [Entity::Component] on the local namespace, used when
  /// the namespace is irrelevant. Caution: this is not portable.
  pub fn local_component<T: AsRef<str>>(name: T) -> Self {
    Self::Component(Self::LOCAL.to_owned(), name.as_ref().to_owned())
  }

  /// Constructor for [Entity::Component] without a namespace, used when
  /// the namespace is irrelevant. Caution: this is not portable.
  #[deprecated(note = "please use `local_component()` instead")]
  pub fn component_direct<T: AsRef<str>>(name: T) -> Self {
    Self::Component(Self::LOCAL.to_owned(), name.as_ref().to_owned())
  }

  /// Constructor for Entity::System.
  pub fn system<T: AsRef<str>, U: AsRef<str>>(name: T, value: U) -> Self {
    Self::System(SystemEntity {
      name: name.as_ref().to_owned(),
      value: value.as_ref().to_owned(),
    })
  }

  /// Constructor for Entity::Test.
  pub fn test<T: AsRef<str>>(msg: T) -> Self {
    Self::Test(msg.as_ref().to_owned())
  }

  /// Constructor for Entity::Provider.
  pub fn provider<T: AsRef<str>>(id: T) -> Self {
    Self::Provider(id.as_ref().to_owned())
  }

  /// Constructor for Entity::Schematic.
  pub fn schematic<T: AsRef<str>>(id: T) -> Self {
    Self::Schematic(id.as_ref().to_owned())
  }

  /// Constructor for Entity::Host.
  pub fn host<T: AsRef<str>>(id: T) -> Self {
    Self::Host(id.as_ref().to_owned())
  }

  /// Constructor for Entity::Client.
  pub fn client<T: AsRef<str>>(id: T) -> Self {
    Self::Client(id.as_ref().to_owned())
  }

  /// Constructor for Entity::Client.
  pub fn reference<T: AsRef<str>>(id: T) -> Self {
    Self::Reference(id.as_ref().to_owned())
  }

  /// The URL of the entity.
  #[must_use]
  pub fn url(&self) -> String {
    match self {
      Entity::Test(msg) => format!("{}://test.sys/?msg={}", URL_SCHEME, msg),
      Entity::Schematic(name) => format!("{}://{}.schem/", URL_SCHEME, name),
      Entity::Component(ns, id) => format!("{}://{}.prov/{}", URL_SCHEME, ns, id),
      Entity::Provider(name) => format!("{}://{}.prov/", URL_SCHEME, name),
      Entity::Client(id) => format!("{}://{}.client/", URL_SCHEME, id),
      Entity::Host(id) => format!("{}://{}.host/", URL_SCHEME, id),
      Entity::System(e) => format!("{}://{}.sys/?msg={}", URL_SCHEME, e.name, e.value),
      Entity::Invalid => format!("{}://invalid.sys/", URL_SCHEME),
      Entity::Reference(id) => format!("{}://{}.ref/", URL_SCHEME, id),
    }
  }

  /// The name of the entity.
  #[must_use]
  pub fn name(&self) -> &str {
    match self {
      Entity::Test(_) => "test",
      Entity::Schematic(name) => name,
      Entity::Component(_, id) => id,
      Entity::Provider(name) => name,
      Entity::Client(id) => id,
      Entity::Host(id) => id,
      Entity::System(e) => &e.name,
      Entity::Invalid => "<invalid>",
      Entity::Reference(id) => id,
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn test() -> Result<(), Error> {
    let entity = Entity::from_str("ofp://namespace.prov/comp_name")?;
    assert_eq!(entity, Entity::component("namespace", "comp_name"));

    let entity = Entity::from_str("ofp://schem_id.schem/")?;
    assert_eq!(entity, Entity::schematic("schem_id"));

    let entity = Entity::from_str("ofp://prov_ns.prov/")?;
    assert_eq!(entity, Entity::provider("prov_ns"));

    let entity = Entity::from_str("ofp://host_id.host/")?;
    assert_eq!(entity, Entity::host("host_id"));

    let entity = Entity::from_str("ofp://host_id.ref/")?;
    assert_eq!(entity, Entity::reference("host_id"));

    let entity = Entity::from_str("ofp://client_id.client/")?;
    assert_eq!(entity, Entity::client("client_id"));

    let entity = Entity::from_str("ofp://test.sys/?msg=Hello")?;
    assert_eq!(entity, Entity::test("Hello"));

    let entity = Entity::from_str("ofp://other.sys/?msg=Else")?;
    assert_eq!(
      entity,
      Entity::System(SystemEntity {
        name: "other".into(),
        value: "Else".into()
      })
    );

    Ok(())
  }
}
