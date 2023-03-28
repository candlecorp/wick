use core::panic;
use std::path::PathBuf;

use wick_config::component_config::{ComponentKind, CompositeComponentConfiguration};
use wick_config::error::ManifestError;
use wick_config::*;

fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
  let path = PathBuf::from(path);
  WickConfiguration::load_from_file(path)
}

fn load_component(path: &str) -> Result<CompositeComponentConfiguration, ManifestError> {
  Ok(load(path)?.try_component_config()?.try_composite()?.clone())
}

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let component = load_component("./tests/manifests/v1/logger.yaml")?;

  assert_eq!(component.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test]
fn test_types() -> Result<(), ManifestError> {
  let types = load("./tests/manifests/v1/http-types.yaml")?.try_types_config()?;
  assert_eq!(types.types().len(), 6);

  Ok(())
}

#[test_logger::test]
fn test_tests() -> Result<(), ManifestError> {
  let tests = load("./tests/manifests/v1/tests.yaml")?.try_test_config()?;

  assert_eq!(tests.tests().len(), 1);

  Ok(())
}

#[test_logger::test]
fn test_operations() -> Result<(), ManifestError> {
  let component = load_component("./tests/manifests/v1/operations.yaml")?;
  assert_eq!(component.operations().len(), 1);

  Ok(())
}

#[test_logger::test]
fn test_main() -> Result<(), ManifestError> {
  let component = load("./tests/manifests/v1/component.yaml")?.try_component_config()?;

  assert!(matches!(component.component().kind(), ComponentKind::Wasm));

  Ok(())
}

#[test_logger::test]
fn regression_issue_42() -> Result<(), ManifestError> {
  let component = load_component("./tests/manifests/v1/shell-expansion.yaml")?;
  println!("{:?}", component);
  let coll = component.component("test").unwrap();
  if let ComponentDefinition::Wasm(module) = &coll.kind {
    let value = module.permissions.dirs.get("/").unwrap();
    assert_ne!(value, "$PWD");
  } else {
    panic!("wrong collection kind");
  }

  Ok(())
}
