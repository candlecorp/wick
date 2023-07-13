use std::str::FromStr;

use anyhow::Result;
use wick_component::serde_util::enum_repr::StringOrNum;
use wick_component::{json, Value};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct Test {
  value: TestEnum,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
enum TestEnum {
  First,
  Second,
}

impl TryFrom<StringOrNum> for TestEnum {
  type Error = String;

  fn try_from(value: StringOrNum) -> std::result::Result<Self, String> {
    match value {
      StringOrNum::String(v) => Self::from_str(&v),
      StringOrNum::Int(v) => Self::from_str(&v.to_string()),
      StringOrNum::Float(v) => Self::from_str(&v.to_string()),
    }
  }
}

impl From<TestEnum> for String {
  fn from(value: TestEnum) -> Self {
    value.value().to_owned()
  }
}

impl TestEnum {
  fn value(&self) -> &str {
    match self {
      TestEnum::First => "1",
      TestEnum::Second => "2",
    }
  }
}

impl FromStr for TestEnum {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "1" => Ok(TestEnum::First),
      "2" => Ok(TestEnum::Second),
      _ => Err(s.to_owned()),
    }
  }
}

impl std::fmt::Display for TestEnum {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value())
  }
}

#[test_logger::test(tokio::test)]
async fn enum_repr() -> Result<()> {
  let expected = Test { value: TestEnum::First };
  let bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected)?;
  let expected_good_json_str = json!({"value": "1"});
  let expected_good_json_num = json!({"value": 1});

  let actual_json: Value = wick_component::wasmrs_codec::messagepack::deserialize(&bytes)?;

  let actual = wick_component::wasmrs_codec::messagepack::deserialize(&bytes)?;
  assert_eq!(expected, actual);

  let actual_json_bytes = wick_component::wasmrs_codec::messagepack::serialize(&actual_json)?;
  let actual_from_json = wick_component::wasmrs_codec::messagepack::deserialize(&actual_json_bytes)?;
  assert_eq!(expected, actual_from_json);

  let expected_json_str_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected_good_json_str)?;
  let expected_json_str_rt = wick_component::wasmrs_codec::messagepack::deserialize(&expected_json_str_bytes)?;
  assert_eq!(expected, expected_json_str_rt);

  let expected_json_num_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected_good_json_num)?;
  let expected_json_num_rt = wick_component::wasmrs_codec::messagepack::deserialize(&expected_json_num_bytes)?;
  assert_eq!(expected, expected_json_num_rt);

  Ok(())
}
