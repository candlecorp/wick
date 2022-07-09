use std::collections::HashMap;

use serde_json::Value;

pub(crate) fn deserialize_json_env<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
  D: serde::de::Deserializer<'de>,
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

fn expand(value: String) -> Result<String, shellexpand::LookupError<std::env::VarError>> {
  Ok(shellexpand::env(&value)?.into_owned())
}

pub(crate) fn kv_deserializer<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
  D: serde::de::Deserializer<'de>,
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
