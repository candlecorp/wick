use std::borrow::Cow;

use wick_wascap::{Token, WickComponent};

use crate::error::WasmComponentError;

#[derive(Clone, Debug)]
pub(crate) struct WickWasmModule<'a> {
  pub(crate) token: Token<WickComponent>,
  pub(crate) bytes: Cow<'a, [u8]>,
}

impl<'a> WickWasmModule<'a> {
  /// Create a component from the bytes of a signed WebAssembly module. Attempting to load.
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub(crate) fn from_vec(buf: Vec<u8>) -> Result<WickWasmModule<'a>, WasmComponentError> {
    let token = wick_wascap::extract_claims(&buf).map_err(|e| WasmComponentError::ClaimsError(e.to_string()))?;
    token.map_or(Err(WasmComponentError::ClaimsExtraction), |t| {
      Ok(WickWasmModule {
        token: t,
        bytes: Cow::Owned(buf),
      })
    })
  }
}
