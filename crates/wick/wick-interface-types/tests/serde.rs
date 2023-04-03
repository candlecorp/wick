use std::fs::read_to_string;

use anyhow::Result;
use pretty_assertions::assert_eq;
use wick_interface_types::{
  ComponentSignature,
  EnumSignature,
  EnumVariant,
  Field,
  InternalType,
  OperationSignature,
  TypeDefinition,
  TypeSignature,
};

#[test_log::test]
fn test_deserialize() -> Result<()> {
  let src = read_to_string("./tests/testdata/interface.json")?;

  let sig: ComponentSignature = serde_json::from_str(&src)?;
  assert_eq!(sig.name, Some("blog".to_owned()));
  let as_json = serde_json::to_string(&sig)?;
  let actual_as_value: serde_json::Value = serde_json::from_str(&as_json)?;

  let expected_as_value: serde_json::Value = serde_json::from_str(&src)?;

  assert_eq!(actual_as_value, expected_as_value);

  Ok(())
}

#[test_log::test]
fn test_deserialize_yaml() -> Result<()> {
  let src = read_to_string("./tests/testdata/http-types.yaml")?;

  let sig: crate::ComponentSignature = serde_yaml::from_str(&src)?;
  let rt = serde_yaml::to_string(&sig)?;

  assert_eq!(sig.types.len(), 6);

  let actual_as_value: crate::ComponentSignature = serde_yaml::from_str(&rt)?;
  let expected_as_value: crate::ComponentSignature = serde_yaml::from_str(&src)?;

  assert_eq!(actual_as_value, expected_as_value);

  Ok(())
}

#[test_log::test]
fn test_deserialize2() -> Result<()> {
  let src = read_to_string("./tests/testdata/interface-test.json")?;

  let sig: ComponentSignature = serde_json::from_str(&src)?;
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
    TypeSignature::Bytes,
    TypeSignature::String,
    TypeSignature::Datetime,
    TypeSignature::Map {
      key: Box::new(TypeSignature::String),
      value: Box::new(TypeSignature::I32),
    },
    TypeSignature::List {
      ty: Box::new(TypeSignature::String),
    },
    TypeSignature::Ref {
      reference: "ref-test".to_owned(),
    },
    TypeSignature::Optional {
      ty: Box::new(TypeSignature::String),
    },
    TypeSignature::Link {
      schemas: vec!["link-test".to_owned()],
    },
  ];

  let json = serde_json::to_string(&types)?;
  println!("{}", json);

  Ok(())
}

#[test_log::test]
fn test_serde_rt() -> Result<()> {
  let mut sig = ComponentSignature::new("test-sig");
  sig.types.push(TypeDefinition::Enum(EnumSignature::new(
    "Unit",
    vec![
      EnumVariant::new("millis", Some(0), None),
      EnumVariant::new("micros", Some(1), None),
    ],
  )));
  let mut compsig = OperationSignature::new("my_component");
  compsig.inputs.push(Field::new("input1", TypeSignature::String));
  compsig.inputs.push(Field::new("input2", TypeSignature::U64));
  compsig.outputs.push(Field::new("output1", TypeSignature::String));
  compsig.outputs.push(Field::new("output2", TypeSignature::U64));
  compsig.outputs.push(Field::new(
    "output2",
    TypeSignature::Ref {
      reference: "MYREF".to_owned(),
    },
  ));

  sig.operations.push(compsig);

  let json = serde_json::to_string(&sig)?;
  eprintln!("{}", json);
  let actual: ComponentSignature = serde_json::from_str(&json)?;
  assert_eq!(actual, sig);

  Ok(())
}

#[test_log::test]
fn test_serde_yaml_rt() -> Result<()> {
  let mut sig = ComponentSignature::new("test-sig");
  sig.types.push(TypeDefinition::Enum(EnumSignature::new(
    "Unit",
    vec![
      EnumVariant::new("millis", Some(0), None),
      EnumVariant::new("micros", Some(1), None),
    ],
  )));
  let mut compsig = OperationSignature::new("my_component");
  compsig.inputs.push(Field::new("input1", TypeSignature::String));
  compsig.inputs.push(Field::new("input2", TypeSignature::U64));
  compsig.inputs.push(Field::new(
    "map1",
    TypeSignature::Map {
      key: Box::new(TypeSignature::String),
      value: Box::new(TypeSignature::String),
    },
  ));
  compsig.inputs.push(Field::new(
    "list1",
    TypeSignature::List {
      ty: Box::new(TypeSignature::String),
    },
  ));

  compsig.outputs.push(Field::new("output1", TypeSignature::String));
  compsig.outputs.push(Field::new("output2", TypeSignature::U64));
  compsig.outputs.push(Field::new(
    "output2",
    TypeSignature::Ref {
      reference: "MYREF".to_owned(),
    },
  ));

  sig.operations.push(compsig);

  let json = serde_yaml::to_string(&sig)?;
  eprintln!("{}", json);
  let actual: ComponentSignature = serde_yaml::from_str(&json)?;
  assert_eq!(actual, sig);

  Ok(())
}
