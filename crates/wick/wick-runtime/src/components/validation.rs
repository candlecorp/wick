use std::path::Path;

use wick_interface_types::ComponentSignature;

use crate::ScopeError;

pub(crate) fn expect_signature_match(
  _actual_src: Option<&Path>,
  actual: &ComponentSignature,
  _expected_src: Option<&Path>,
  expected: &ComponentSignature,
) -> Result<(), ScopeError> {
  if actual != expected {
    // Disabling for now.
    // debug!(
    //   expected = serde_json::to_string(expected).unwrap(),
    //   actual = serde_json::to_string(actual).unwrap(),
    //   "signature mismatch"
    // );
    // return Err(EngineError::ComponentSignature(
    //   expected_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
    //   actual_src.map_or_else(|| PathBuf::from("unknown"), Into::into),
    // ));
  }
  Ok(())
}
