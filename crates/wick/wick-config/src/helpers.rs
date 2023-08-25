use std::collections::HashMap;

use serde::Deserializer;

#[allow(unused)]
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
        map.insert(key, value);
      }
      Ok(map)
    }
  }

  deserializer.deserialize_any(HashMapVisitor)
}
