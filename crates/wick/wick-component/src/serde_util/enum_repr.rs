#[derive(Debug, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
#[serde(untagged)]
/// A string or number to attempt deserializing from.
#[allow(missing_docs, clippy::exhaustive_enums)]
pub enum StringOrNum {
  String(String),
  Int(i64),
  Float(f64),
}

/// Serialize a value as a string using its [std::fmt::Display] implementation.
pub fn serialize<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
  T: std::fmt::Display,
{
  let s = val.to_string();
  serializer.serialize_str(&s)
}

/// Deserialize a value from a number or string using its [std::str::FromStr] implementation.
pub fn deserialize<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
  S: std::str::FromStr,
  S::Err: std::fmt::Display,
  D: serde::Deserializer<'de>,
{
  let s: StringOrNum = serde::Deserialize::deserialize(deserializer)?;
  let s = match s {
    StringOrNum::String(s) => s,
    StringOrNum::Int(i) => i.to_string(),
    StringOrNum::Float(i) => i.to_string(),
  };
  S::from_str(&s).map_err(serde::de::Error::custom)
}
