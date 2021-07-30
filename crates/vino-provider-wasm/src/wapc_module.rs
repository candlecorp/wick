use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use vino_wascap::{
  Claims,
  ComponentClaims,
  Token,
};

use crate::error::WasmProviderError;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WapcModule {
  pub token: Token<ComponentClaims>,
  pub bytes: Vec<u8>,
}

impl WapcModule {
  /// Create an actor from the bytes of a signed WebAssembly module. Attempting to load
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub fn from_slice(buf: &[u8]) -> Result<WapcModule, WasmProviderError> {
    let token = vino_wascap::extract_claims(&buf)?;
    token.map_or(
      Err(WasmProviderError::ClaimsError(
        "Could not extract claims from component".to_owned(),
      )),
      |t| {
        Ok(WapcModule {
          token: t,
          bytes: buf.to_vec(),
        })
      },
    )
  }

  /// Create an actor from a signed WebAssembly (`.wasm`) file.
  pub fn from_file(path: &Path) -> Result<WapcModule, WasmProviderError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    WapcModule::from_slice(&buf).map_err(|_| WasmProviderError::FileNotFound(path.to_path_buf()))
  }

  /// Obtain the issuer's public key as it resides in the actor's token (the `iss` field of the JWT).
  #[must_use]
  pub fn _issuer(&self) -> String {
    self.token.claims.issuer.clone()
  }

  /// Obtain the list of tags in the actor's token.
  #[must_use]
  pub fn _tags(&self) -> Vec<String> {
    match self.token.claims.metadata.as_ref().unwrap().tags {
      Some(ref tags) => tags.clone(),
      None => vec![],
    }
  }

  /// Obtain the actor's public key (The `sub` field of the JWT).
  #[must_use]
  pub fn public_key(&self) -> String {
    self.token.claims.subject.clone()
  }

  /// A globally referencable ID to this component
  #[must_use]
  pub fn id(&self) -> String {
    self.public_key()
  }

  /// The actor's human-friendly display name
  #[must_use]
  pub fn name(&self) -> String {
    self
      .token
      .claims
      .metadata
      .as_ref()
      .unwrap()
      .interface
      .name
      .clone()
  }

  // Obtain the raw set of claims for this actor.
  #[must_use]
  pub fn claims(&self) -> Claims<ComponentClaims> {
    self.token.claims.clone()
  }
}
