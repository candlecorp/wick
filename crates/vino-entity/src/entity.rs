use std::fmt::Display;
use std::str::FromStr;

use serde::{
  Deserialize,
  Serialize,
};

use crate::error::EntityError as Error;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The entity being referenced across systems or services.
pub enum Entity {
  /// A system entity with the name "invalid". Used only for situations where a default is more useful than an error.
  Invalid,
  /// The system entity is used when communicating to or from the internals of another component. Used mostly by library developers.
  System(SystemEntity),
  /// A system entity with the name "test". Used as the originating entity for tests.
  Test(String),
  /// A client entity used for requests.
  Client(String),
  /// A Host entity used for entities that serve responses to requests.
  Host(String),
  /// A schematic
  Schematic(String),
  /// A component or anything that can be invoked like a component
  Component(ComponentEntity),
  /// A provider (an entity that hosts a collection of components)
  Provider(String),
  /// A reference to an instance of an entity.
  Reference(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemEntity {
  name: String,
  value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentEntity {
  pub reference: String,
  pub name: String,
}

impl Display for ComponentEntity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}({})", self.name, self.reference)
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

pub(crate) const URL_SCHEME: &str = "ofp";

impl FromStr for Entity {
  type Err = Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
      .split_once(".")
      .ok_or_else(|| Error::ParseError(format!("Invalid authority format '{}', no dot.", host)))?;
    match kind {
      "system" => {
        let (_, msg) = url
          .query_pairs()
          .find(|(k, _v)| k == "msg")
          .unwrap_or(("".into(), "".into()));

        if id == "test" {
          Ok(Entity::Test(msg.into()))
        } else {
          Ok(Entity::System(SystemEntity {
            name: id.into(),
            value: msg.into(),
          }))
        }
      }
      "schematic" => Ok(Entity::Schematic(id.into())),
      "component" => {
        let mut segments = url.path_segments().ok_or_else(|| {
          Error::ParseError(format!(
            "Invalid component URL, no path segments found in '{}'",
            url
          ))
        })?;
        let reference = segments.next().ok_or_else(|| {
          Error::ParseError(format!(
            "Invalid component URL, no reference path segment found in '{}'",
            url
          ))
        })?;
        Ok(Entity::Component(ComponentEntity {
          name: id.into(),
          reference: reference.into(),
        }))
      }
      "provider" => Ok(Entity::Provider(id.into())),
      "client" => Ok(Entity::Client(id.into())),
      "host" => Ok(Entity::Host(id.into())),
      _ => Err(Error::ParseError(format!(
        "Invalid authority kind: {}",
        kind
      ))),
    }
  }
}
impl Entity {
  pub fn component(name: &str, reference: &str) -> Self {
    Self::Component(ComponentEntity {
      name: name.to_owned(),
      reference: reference.to_owned(),
    })
  }

  pub fn system(name: &str, value: &str) -> Self {
    Self::System(SystemEntity {
      name: name.to_owned(),
      value: value.to_owned(),
    })
  }

  pub fn test(msg: &str) -> Self {
    Self::Test(msg.to_owned())
  }

  pub fn provider(id: &str) -> Self {
    Self::Provider(id.to_owned())
  }

  pub fn schematic(id: &str) -> Self {
    Self::Schematic(id.to_owned())
  }

  pub fn host(id: &str) -> Self {
    Self::Host(id.to_owned())
  }

  pub fn client(id: &str) -> Self {
    Self::Client(id.to_owned())
  }

  /// The URL of the entity
  #[must_use]
  pub fn url(&self) -> String {
    match self {
      Entity::Test(msg) => format!("{}://test.system/?msg={}", URL_SCHEME, msg),
      Entity::Schematic(name) => format!("{}://{}.schematic/", URL_SCHEME, name),
      Entity::Component(e) => format!("{}://{}.component/{}", URL_SCHEME, e.name, e.reference),
      Entity::Provider(name) => format!("{}://{}.provider/", URL_SCHEME, name),
      Entity::Client(id) => format!("{}://{}.client/", URL_SCHEME, id),
      Entity::Host(id) => format!("{}://{}.host/", URL_SCHEME, id),
      Entity::System(e) => format!("{}://{}.system/?msg={}", URL_SCHEME, e.name, e.value),
      Entity::Invalid => format!("{}://invalid.system/", URL_SCHEME),
      Entity::Reference(id) => format!("{}://{}.ref/", URL_SCHEME, id),
    }
  }

  /// The unique (public) key of the entity
  #[must_use]
  pub fn key(&self) -> String {
    match self {
      Entity::Test(msg) => format!("system:test:{}", msg),
      Entity::Schematic(name) => format!("schematic:{}", name),
      Entity::Component(e) => format!("component:{}:{}", e.name, e.reference),
      Entity::Provider(name) => format!("provider:{}", name),
      Entity::Client(id) => format!("client:{}", id),
      Entity::Host(id) => format!("host:{}", id),
      Entity::System(e) => format!("system:{}:{}", e.name, e.value),
      Entity::Invalid => "system:invalid".to_owned(),
      Entity::Reference(id) => format!("reference:{}", id),
    }
  }

  pub fn into_provider(self) -> Result<String> {
    match self {
      Entity::Provider(s) => Ok(s),
      _ => Err(Error::ConversionError("into_provider")),
    }
  }

  pub fn into_component(self) -> Result<ComponentEntity> {
    match self {
      Entity::Component(s) => Ok(s),
      _ => Err(Error::ConversionError("into_component")),
    }
  }

  pub fn into_schematic(self) -> Result<String> {
    match self {
      Entity::Schematic(s) => Ok(s),
      _ => Err(Error::ConversionError("into_schematic")),
    }
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq as equals;

  use super::*;
  #[test]
  fn test() -> Result<()> {
    let entity = Entity::from_str("ofp://some_id.component/name/reference")?;
    equals!(
      entity,
      Entity::Component(ComponentEntity {
        name: "name".into(),
        reference: "reference".into(),
      })
    );

    let entity = Entity::from_str("ofp://some_id.schematic/")?;
    equals!(entity, Entity::Schematic("some_id".into()));

    let entity = Entity::from_str("ofp://some_id.provider/")?;
    equals!(entity, Entity::Provider("some_id".into()));

    let entity = Entity::from_str("ofp://some_id.host/")?;
    equals!(entity, Entity::Host("some_id".into()));

    let entity = Entity::from_str("ofp://some_id.client/")?;
    equals!(entity, Entity::Client("some_id".into()));

    let entity = Entity::from_str("ofp://test.system/msg=Hello")?;
    equals!(entity, Entity::Test("Hello".into()));

    let entity = Entity::from_str("ofp://other.system/msg=Else")?;
    equals!(
      entity,
      Entity::System(SystemEntity {
        name: "other".into(),
        value: "Else".into()
      })
    );

    Ok(())
  }
}
