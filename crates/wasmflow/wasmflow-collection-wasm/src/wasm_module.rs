use std::path::Path;

use wasmflow_wascap::{Claims, CollectionClaims, Token};

use crate::error::WasmCollectionError;

#[derive(Clone, Debug)]
pub struct WickWasmModule {
  pub token: Token<CollectionClaims>,
  pub bytes: Vec<u8>,
}

impl WickWasmModule {
  /// Create an actor from the bytes of a signed WebAssembly module. Attempting to load.
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub fn from_slice(buf: &[u8]) -> Result<WickWasmModule, WasmCollectionError> {
    let token = wasmflow_wascap::extract_claims(buf).map_err(|e| WasmCollectionError::ClaimsError(e.to_string()))?;
    token.map_or(Err(WasmCollectionError::ClaimsExtraction), |t| {
      Ok(WickWasmModule {
        token: t,
        bytes: buf.to_vec(),
      })
    })
  }

  /// Create an actor from a signed WebAssembly (`.wasm`) file.
  pub async fn from_file(path: &Path) -> Result<WickWasmModule, WasmCollectionError> {
    let file = tokio::fs::read(path).await?;

    WickWasmModule::from_slice(&file)
  }

  /// Obtain the actor's public key (The `sub` field of the JWT).
  #[must_use]
  pub fn public_key(&self) -> &String {
    &self.token.claims.subject
  }

  /// A globally referencable ID to this component.
  #[must_use]
  pub fn id(&self) -> &String {
    self.public_key()
  }

  /// The actor's human-friendly display name.
  #[must_use]
  pub fn name(&self) -> &Option<String> {
    &self.token.claims.metadata.as_ref().unwrap().interface.name
  }

  // Obtain the raw set of claims for this component.
  #[must_use]
  pub fn claims(&self) -> &Claims<CollectionClaims> {
    &self.token.claims
  }
}
