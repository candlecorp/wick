use std::path::PathBuf;

use wasmflow_manifest::error::ManifestError;
use wasmflow_manifest::*;

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v1/logger.yaml");
  let manifest = WasmflowManifest::load_from_file(&path)?;

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}
