#![cfg(feature = "parser")]
use std::collections::HashMap;

use anyhow::Result;
use wasmflow_interface::{
  component,
  operation,
  parse,
  typemap,
  CollectionFeatures,
  CollectionSignature,
  CollectionVersion,
  OperationMap,
  OperationSignature,
  TypeMap,
  TypeSignature,
};

#[test_log::test]
fn test_parser() -> Result<()> {
  assert_eq!(parse("bool")?, TypeSignature::Bool);
  assert_eq!(
    parse("bool[]")?,
    TypeSignature::List {
      element: Box::new(TypeSignature::Bool),
    }
  );
  let fields: HashMap<String, TypeSignature> = [("myBool".to_owned(), TypeSignature::Bool)].into();
  assert_eq!(
    parse("{ myBool : bool }")?,
    TypeSignature::AnonymousStruct(fields.into())
  );

  Ok(())
}

#[test_log::test]
fn test_typemap() -> Result<()> {
  let map = typemap! {
    "myBool" => "bool",
  };
  let fields: HashMap<String, TypeSignature> = [("myBool".to_owned(), TypeSignature::Bool)].into();
  assert_eq!(map, fields.into());

  Ok(())
}

#[test_log::test]
fn test_op_macro() -> Result<()> {
  let actual = operation! {
    "test-component" => {
      inputs: {"input"=>"string",},
      outputs: {"output"=>"string",},
    }
  };
  let expected = OperationSignature {
    index: 0,
    name: "test-component".to_owned(),
    inputs: typemap! {"input"=>"string"},
    outputs: typemap! {"output"=>"string"},
  };
  assert_eq!(actual, expected);
  let actual = operation! {"math::subtract" => {
    inputs: { "left" => "u64", "right" => "u64",},
    outputs: { "output" => "u64" ,},
  }};
  let expected = OperationSignature {
    index: 0,
    name: "math::subtract".to_owned(),
    inputs: typemap! {"left"=>"u64","right"=>"u64"},
    outputs: typemap! {"output"=>"u64"},
  };
  assert_eq!(actual, expected);

  Ok(())
}

#[test_log::test]
fn test_component_macro() -> Result<()> {
  let mut opmap = OperationMap::default();
  opmap.insert(
    "test-component",
    operation! {
      "test-component" => {
        inputs: {"input"=>"string"},
        outputs: {"output"=>"string"},
      }
    },
  );

  let expected = CollectionSignature {
    name: Some("test-native-collection".to_owned()),
    features: CollectionFeatures {
      streaming: false,
      stateful: true,
      version: CollectionVersion::V0,
    },
    format: 1,
    version: "0.1.0".to_owned(),
    types: std::collections::HashMap::from([]).into(),
    operations: opmap,
    wellknown: Vec::new(),
    config: TypeMap::new(),
  };
  let actual = component! {
    "test-native-collection" => {
      version: "0.1.0",
      operations: {
        "test-component" => {
          inputs: {"input" => "string"},
          outputs: {"output" => "string"},
        }
      }
    }
  };
  assert_eq!(actual, expected);
  Ok(())
}
