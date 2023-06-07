use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub(crate) fn with_expand_envs_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  let val = String::deserialize(deserializer)?;
  #[allow(clippy::option_if_let_else)]
  match shellexpand::env(&val) {
    Ok(value) => match value.parse::<String>() {
      Ok(value) => Ok(value),
      Err(_) => Ok(value.into_owned()),
    },
    Err(_) => Ok(val),
  }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn deserialize_json_env<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
  D: Deserializer<'de>,
{
  // define a visitor that deserializes
  // `ActualData` encoded as json within a string
  struct JsonStringVisitor;

  impl<'de> serde::de::Visitor<'de> for JsonStringVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a string containing json data")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      let mut map = serde_json::Map::with_capacity(access.size_hint().unwrap_or(0));
      while let Some((key, value)) = access.next_entry()? {
        map.insert(key, expand_jsonval(value).map_err(serde::de::Error::custom)?);
      }
      Ok(Value::Object(map))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::SeqAccess<'de>,
    {
      let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(0));
      while let Some(value) = seq.next_element()? {
        list.push(expand_jsonval(value).map_err(serde::de::Error::custom)?);
      }
      Ok(Value::Array(list))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      let rx = regex::Regex::new(r#"re"#).unwrap();
      rx.replace_all(v, |captures: &regex::Captures| {
        captures
          .iter()
          .map(|c| {
            c.map(|c| shellexpand::env(c.as_str()).unwrap_or_else(|e| panic!("Could not expand env variable: {}", e)))
              .unwrap_or_default()
          })
          .collect::<Vec<_>>()
          .join("")
      });

      // unfortunately we lose some typed information
      // from errors deserializing the json string
      serde_json::from_str(v).map_err(E::custom)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_str(&v)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_i64(v as i64)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_i64(v as i64)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_i64(v as i64)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Err(serde::de::Error::invalid_type(serde::de::Unexpected::Signed(v), &self))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_u64(v as u64)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_u64(v as u64)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_u64(v as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::Number(serde_json::Number::from(v)))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      serde_json::Number::from_f64(v).map_or_else(
        || Err(serde::de::Error::invalid_type(serde::de::Unexpected::Float(v), &self)),
        |n| Ok(Value::Number(n)),
      )
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_str(v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
      D: Deserializer<'de>,
    {
      deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(Value::Null)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::EnumAccess<'de>,
    {
      let _ = data;
      Err(serde::de::Error::invalid_type(serde::de::Unexpected::Enum, &self))
    }
  }

  // use our visitor to deserialize an `ActualValue`
  deserializer.deserialize_any(JsonStringVisitor)
}

fn expand_jsonval(value: Value) -> Result<Value, shellexpand::LookupError<std::env::VarError>> {
  match value {
    Value::String(s) => Ok(Value::String(shellexpand::env(&s)?.into_owned())),
    Value::Array(list) => {
      let list = list.into_iter().map(expand_jsonval).collect::<Result<Vec<_>, _>>()?;
      Ok(Value::Array(list))
    }
    Value::Object(map) => {
      let mut new_map = serde_json::Map::with_capacity(map.len());
      for (k, v) in map {
        new_map.insert(shellexpand::env(&k)?.into_owned(), expand_jsonval(v)?);
      }
      Ok(Value::Object(new_map))
    }
    x => Ok(x),
  }
}

#[allow(clippy::needless_pass_by_value)]
fn expand(value: String) -> Result<String, shellexpand::LookupError<std::env::VarError>> {
  Ok(shellexpand::env(&value)?.into_owned())
}

pub(crate) fn kv_deserializer<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
  D: Deserializer<'de>,
{
  struct HashMapVisitor;

  impl<'de> serde::de::Visitor<'de> for HashMapVisitor {
    type Value = HashMap<String, String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a key/value map")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
      while let Some((key, value)) = access.next_entry()? {
        map.insert(
          expand(key).map_err(serde::de::Error::custom)?,
          expand(value).map_err(serde::de::Error::custom)?,
        );
      }
      Ok(map)
    }
  }

  deserializer.deserialize_any(HashMapVisitor)
}

pub(crate) fn configmap_deserializer<'de, D>(deserializer: D) -> Result<Option<HashMap<String, Value>>, D::Error>
where
  D: Deserializer<'de>,
{
  struct HashMapVisitor;

  impl<'de> serde::de::Visitor<'de> for HashMapVisitor {
    type Value = Option<HashMap<String, Value>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a key/value map or nil")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
      while let Some((key, mut value)) = access.next_entry::<String, Value>()? {
        if let Value::String(s) = value {
          value = Value::String(expand(s).map_err(serde::de::Error::custom)?);
        }

        map.insert(expand(key).map_err(serde::de::Error::custom)?, value);
      }
      Ok(Some(map))
    }
  }

  deserializer.deserialize_any(HashMapVisitor)
}
