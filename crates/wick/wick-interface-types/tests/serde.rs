use std::fs::read_to_string;

use anyhow::Result;
use pretty_assertions::assert_eq;
use wick_interface_types::{
  ComponentSignature,
  EnumDefinition,
  EnumVariant,
  Field,
  OperationSignature,
  Type,
  TypeDefinition,
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
  let types: Vec<Type> = vec![
    Type::Bool,
    Type::I8,
    Type::I16,
    Type::I32,
    Type::I64,
    Type::U8,
    Type::U16,
    Type::U32,
    Type::U64,
    Type::F32,
    Type::F64,
    Type::Bytes,
    Type::String,
    Type::Datetime,
    Type::Map {
      key: Box::new(Type::String),
      value: Box::new(Type::I32),
    },
    Type::List {
      ty: Box::new(Type::String),
    },
    Type::Named("ref-test".to_owned()),
    Type::Optional {
      ty: Box::new(Type::String),
    },
  ];

  let json = serde_json::to_string(&types)?;
  println!("{}", json);

  Ok(())
}

#[test_log::test]
fn test_serde_rt() -> Result<()> {
  let mut sig = ComponentSignature::new_named("test-sig");
  sig.types.push(TypeDefinition::Enum(EnumDefinition::new(
    "Unit",
    vec![
      EnumVariant::new("millis", Some(0), None, None),
      EnumVariant::new("micros", Some(1), None, None),
    ],
    None,
  )));
  let mut compsig = OperationSignature::new_named("my_component");
  compsig.inputs.push(Field::new("input1", Type::String));
  compsig.inputs.push(Field::new("input2", Type::U64));
  compsig.outputs.push(Field::new("output1", Type::String));
  compsig.outputs.push(Field::new("output2", Type::U64));
  compsig
    .outputs
    .push(Field::new("output2", Type::Named("MYREF".to_owned())));

  sig.operations.push(compsig);

  let json = serde_json::to_string(&sig)?;
  eprintln!("{}", json);
  let actual: ComponentSignature = serde_json::from_str(&json)?;
  assert_eq!(actual, sig);

  Ok(())
}

#[test_log::test]
fn test_serde_yaml_rt() -> Result<()> {
  let mut sig = ComponentSignature::new_named("test-sig");
  sig.types.push(TypeDefinition::Enum(EnumDefinition::new(
    "Unit",
    vec![
      EnumVariant::new("millis", Some(0), None, None),
      EnumVariant::new("micros", Some(1), None, None),
    ],
    None,
  )));
  let mut compsig = OperationSignature::new_named("my_component");
  compsig.inputs.push(Field::new("input1", Type::String));
  compsig.inputs.push(Field::new("input2", Type::U64));
  compsig.inputs.push(Field::new(
    "map1",
    Type::Map {
      key: Box::new(Type::String),
      value: Box::new(Type::String),
    },
  ));
  compsig.inputs.push(Field::new(
    "list1",
    Type::List {
      ty: Box::new(Type::String),
    },
  ));

  compsig.outputs.push(Field::new("output1", Type::String));
  compsig.outputs.push(Field::new("output2", Type::U64));
  compsig
    .outputs
    .push(Field::new("output2", Type::Named("MYREF".to_owned())));

  sig.operations.push(compsig);

  let json = serde_yaml::to_string(&sig)?;
  eprintln!("{}", json);
  let actual: ComponentSignature = serde_yaml::from_str(&json)?;
  assert_eq!(actual, sig);

  Ok(())
}
