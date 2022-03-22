use std::fs::read_to_string;

use anyhow::Result;
use vino_types::ProviderSignature;

#[test_log::test]
fn test_deserialize() -> Result<()> {
  let src = read_to_string("./tests/interface.json")?;

  let sig: ProviderSignature = serde_json::from_str(&src)?;
  assert_eq!(sig.name, Some("blog".to_owned()));
  let as_json = serde_json::to_string(&sig)?;
  let actual_as_value: serde_json::Value = serde_json::from_str(&as_json)?;

  let expected_as_value: serde_json::Value = serde_json::from_str(&src)?;

  assert_eq!(actual_as_value, expected_as_value);

  Ok(())
}

#[test_log::test]
fn test_deserialize2() -> Result<()> {
  let src = read_to_string("./tests/interface-test.json")?;

  let sig: ProviderSignature = serde_json::from_str(&src)?;
  assert_eq!(sig.name, Some("test-component".to_owned()));
  let as_json = serde_json::to_string(&sig)?;
  let actual_as_value: serde_json::Value = serde_json::from_str(&as_json)?;

  let expected_as_value: serde_json::Value = serde_json::from_str(&src)?;

  assert_eq!(actual_as_value, expected_as_value);

  Ok(())
}
