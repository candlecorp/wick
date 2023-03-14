use std::path::PathBuf;

use wick_config::error::ManifestError;
use wick_config::*;

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/logger.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;
  println!("{:?}", manifest);

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test]
fn test_types() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/http-types.yaml");
  let component = ComponentConfiguration::load_from_file(path)?;

  assert_eq!(component.types().len(), 6);

  Ok(())
}

#[test_logger::test]
fn regression_issue_42() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/shell-expansion.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;
  println!("{:?}", manifest);
  let coll = manifest.component("test").unwrap();
  if let ComponentKind::Wasm(module) = &coll.kind {
    let value = module.permissions.dirs.get("/").unwrap();
    assert_ne!(value, "$PWD");
  } else {
    panic!("wrong collection kind");
  }

  Ok(())
}
