use std::path::PathBuf;

use wasmflow_manifest::error::ManifestError;
use wasmflow_manifest::*;

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/logger.wafl");
  let manifest = WasmflowManifest::load_from_file(&path)?;
  println!("{:?}", manifest);

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test]
fn regression_issue_42() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/shell-expansion.wafl");
  let manifest = WasmflowManifest::load_from_file(&path)?;
  println!("{:?}", manifest);
  let coll = manifest.collection("test").unwrap();
  if let CollectionKind::Wasm(module) = &coll.kind {
    let value = module.permissions.dirs.get("/").unwrap();
    assert_ne!(value, "$PWD");
  } else {
    panic!("wrong collection kind");
  }

  Ok(())
}
