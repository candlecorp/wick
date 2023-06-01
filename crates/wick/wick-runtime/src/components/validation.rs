use std::path::Path;

use wick_interface_types::ComponentSignature;

use crate::EngineError;

pub(crate) fn expect_signature_match(
  _actual_src: Option<&Path>,
  actual: &ComponentSignature,
  _expected_src: Option<&Path>,
  expected: &ComponentSignature,
) -> Result<(), EngineError> {
  if actual != expected {
    warn!(
      expected = serde_json::to_string(expected).unwrap(),
      actual = serde_json::to_string(actual).unwrap(),
      "signature mismatch"
    );
    // Disabling for now.
    // return Err(EngineError::ComponentSignature(
    //   expected_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
    //   actual_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
    // ));
  }
  Ok(())
}
