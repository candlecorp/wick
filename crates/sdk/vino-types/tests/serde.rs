use std::fs::read_to_string;

use anyhow::Result;
use pretty_assertions::assert_eq;
use vino_types::{InternalType, ProviderSignature, TypeSignature};

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

#[test_log::test]
fn test_serde_all() -> Result<()> {
  let types: Vec<TypeSignature> = vec![
    TypeSignature::Bool,
    TypeSignature::I8,
    TypeSignature::I16,
    TypeSignature::I32,
    TypeSignature::I64,
    TypeSignature::U8,
    TypeSignature::U16,
    TypeSignature::U32,
    TypeSignature::U64,
    TypeSignature::F32,
    TypeSignature::F64,
    TypeSignature::Value,
    TypeSignature::Raw,
    TypeSignature::Bytes,
    TypeSignature::String,
    TypeSignature::Datetime,
    TypeSignature::Map {
      key: Box::new(TypeSignature::String),
      value: Box::new(TypeSignature::I32),
    },
    TypeSignature::List {
      element: Box::new(TypeSignature::String),
    },
    TypeSignature::Internal(InternalType::ComponentInput),
    TypeSignature::Ref {
      reference: "ref-test".to_owned(),
    },
    TypeSignature::Optional {
      option: Box::new(TypeSignature::String),
    },
    TypeSignature::Link {
      provider: Some("link-test".to_owned()),
    },
  ];

  let json = serde_json::to_string(&types)?;
  println!("{}", json);

  Ok(())
}
