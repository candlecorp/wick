use std::io::Read;
use std::path::Path;

use tar::Header;
use wasmflow_interface::CollectionSignature;
use wasmflow_wascap::{ClaimsOptions, KeyPair};

use crate::error::ParError;

/// Make a collection archive for the passed binary and signature
pub fn make_archive<T: Read>(
  mut binary: T,
  signature: &CollectionSignature,
  claims_options: ClaimsOptions,
  subject_kp: &KeyPair,
  issuer_kp: &KeyPair,
) -> Result<Vec<u8>, ParError> {
  let signature_json = serde_json::to_string(&signature)?;
  let signature_bytes = signature_json.as_bytes();
  debug!(len = signature_bytes.len(), "bundle signature length");

  let mut sig_header = Header::new_gnu();
  sig_header.set_path("interface.json")?;
  sig_header.set_size(signature_bytes.len().try_into().unwrap());
  sig_header.set_cksum();
  let archive_bytes = Vec::new();
  let mut archive = tar::Builder::new(archive_bytes);

  archive.append(&sig_header, signature_bytes)?;

  let claims = wasmflow_wascap::build_collection_claims(signature.clone(), subject_kp, issuer_kp, claims_options);
  debug!(?claims, "oci archive claims");
  let mut bin_bytes = Vec::new();
  binary.read_to_end(&mut bin_bytes)?;
  let combined_bytes = bin_bytes.chain(&*signature_bytes);
  let jwt_bytes = wasmflow_wascap::make_jwt(combined_bytes, &claims, issuer_kp)?;
  let mut jwt_header = Header::new_gnu();
  jwt_header.set_path("archive.jwt")?;
  jwt_header.set_size(jwt_bytes.len().try_into().unwrap());
  jwt_header.set_cksum();
  debug!(len = jwt_bytes.len(), "jwt length");
  debug!(len = jwt_bytes.len(), "bundle jwt length",);
  archive.append(&jwt_header, &*jwt_bytes)?;

  let mut bin_header = Header::new_gnu();
  bin_header.set_path("main.bin")?;
  bin_header.set_mode(0o555);
  bin_header.set_size(bin_bytes.len() as _);
  bin_header.set_cksum();
  archive.append(&bin_header, &*bin_bytes)?;

  archive.finish()?;
  let archive_bytes = archive.into_inner()?;
  Ok(archive_bytes)
}

/// Path to the JWT in the archive.
pub const JWT_PATH: &str = "archive.jwt";
/// Path to the binary in the archive.
pub const BIN_PATH: &str = "main.bin";
/// Path to the interface in the archive.
pub const INTERFACE_PATH: &str = "interface.json";

/// Validates an archive's contents by decoding the contained JWT and validating its hash against the binary and interface.
pub fn validate_collection<B: Read, I: Read>(binary: B, interface: I, jwt: Vec<u8>) -> Result<(), ParError> {
  let combined = binary.chain(interface);

  let token = wasmflow_wascap::decode_token(jwt)?;
  let hash = wasmflow_wascap::hash_bytes(combined)?;
  trace!(hash = %hash, ?token, "oci archive");

  wasmflow_wascap::assert_valid_jwt(&token, &hash)?;
  Ok(())
}

/// Validates an archive's contents by decoding the contained JWT and validating its hash against the binary and interface.
pub fn validate_collection_dir(dir: &Path) -> Result<(), ParError> {
  let jwt = std::fs::read(dir.join(JWT_PATH)).map_err(|_| ParError::MissingJwt)?;
  let interface = std::fs::File::open(dir.join(INTERFACE_PATH)).map_err(|_| ParError::MissingJwt)?;
  let bin = std::fs::File::open(dir.join(BIN_PATH)).map_err(|_| ParError::MissingJwt)?;
  validate_collection(bin, interface, jwt)
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use tar::Archive;

  use super::*;

  #[test_logger::test]
  fn test_archive_validation() -> Result<()> {
    let signature = CollectionSignature::default();
    let bin_bytes = b"0123456".to_vec();
    let claims = ClaimsOptions::default();
    let subject_kp = KeyPair::new_module();
    let issuer_kp = KeyPair::new_user();
    let archive_bytes = make_archive(&*bin_bytes, &signature, claims, &subject_kp, &issuer_kp)?;
    let mut archive = Archive::new(&*archive_bytes);
    let tmpdir = std::env::temp_dir().join("wafltest");
    archive.unpack(&tmpdir)?;

    validate_collection_dir(&tmpdir)?;

    Ok(())
  }
}
