use std::path::Path;

use wick_wascap::{Claims, Token, WickComponent};

use crate::error::WasmComponentError;

#[derive(Clone, Debug)]
pub struct WickWasmModule {
  pub token: Token<WickComponent>,
  pub bytes: Vec<u8>,
}

impl WickWasmModule {
  /// Create a component from the bytes of a signed WebAssembly module. Attempting to load.
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub fn from_slice(buf: &[u8]) -> Result<WickWasmModule, WasmComponentError> {
    let token = wick_wascap::extract_claims(buf).map_err(|e| WasmComponentError::ClaimsError(e.to_string()))?;
    token.map_or(Err(WasmComponentError::ClaimsExtraction), |t| {
      Ok(WickWasmModule {
        token: t,
        bytes: buf.to_vec(),
      })
    })
  }

  /// Create a component from a signed WebAssembly (`.wasm`) file.
  pub async fn from_file(path: &Path) -> Result<WickWasmModule, WasmComponentError> {
    let file = tokio::fs::read(path).await?;

    WickWasmModule::from_slice(&file)
  }

  /// Obtain the component's public key (The `sub` field of the JWT).
  #[must_use]
  pub fn public_key(&self) -> &String {
    &self.token.claims.subject
  }

  /// A globally referencable ID to this component.
  #[must_use]
  pub fn id(&self) -> &String {
    self.public_key()
  }

  /// The component's human-friendly display name.
  #[must_use]
  pub fn name(&self) -> &Option<String> {
    &self.token.claims.metadata.as_ref().unwrap().interface.name
  }

  /// The component's human-friendly display name.
  #[must_use]
  pub fn hash(&self) -> &String {
    &self.token.claims.metadata.as_ref().unwrap().module_hash
  }

  // Obtain the raw set of claims for this component.
  #[must_use]
  pub fn claims(&self) -> &Claims<WickComponent> {
    &self.token.claims
  }
}
