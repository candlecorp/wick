use std::io::Read;
use std::path::Path;

use tar::Header;
use vino_types::ProviderSignature;
use vino_wascap::{ClaimsOptions, KeyPair};

use crate::error::ParError;

/// Make a provider archive for the passed binary and signature
pub fn make_archive<T: Read>(
  mut binary: T,
  signature: &ProviderSignature,
  claims_options: ClaimsOptions,
  subject_kp: &KeyPair,
  issuer_kp: &KeyPair,
) -> Result<Vec<u8>, ParError> {
  let signature_json = serde_json::to_string(&signature)?;
  let signature_bytes = signature_json.as_bytes();
  debug!("OCI:BUNDLE:SIGNATURE[{} bytes]", signature_bytes.len());

  let mut sig_header = Header::new_gnu();
  sig_header.set_path("interface.json".to_owned())?;
  sig_header.set_size(signature_bytes.len().try_into().unwrap());
  sig_header.set_cksum();
  let archive_bytes = Vec::new();
  let mut archive = tar::Builder::new(archive_bytes);

  archive.append(&sig_header, signature_bytes)?;

  let claims = vino_wascap::build_provider_claims(signature.clone(), subject_kp, issuer_kp, claims_options);
  trace!("OCI:ARCHIVE:CLAIMS:{:?}", claims);
  let mut bin_bytes = Vec::new();
  binary.read_to_end(&mut bin_bytes)?;
  let combined_bytes = bin_bytes.chain(&*signature_bytes);
  let jwt_bytes = vino_wascap::make_jwt(combined_bytes, &claims, issuer_kp)?;
  let mut jwt_header = Header::new_gnu();
  jwt_header.set_path("archive.jwt".to_owned())?;
  jwt_header.set_size(jwt_bytes.len().try_into().unwrap());
  jwt_header.set_cksum();
  debug!("OCI:BUNDLE:JWT[{} bytes]", jwt_bytes.len());
  archive.append(&jwt_header, &*jwt_bytes)?;

  let mut bin_header = Header::new_gnu();
  bin_header.set_path("main.bin".to_owned())?;
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
pub fn validate_provider<B: Read, I: Read>(binary: B, interface: I, jwt: Vec<u8>) -> Result<(), ParError> {
  debug!("OCI:VALIDATE_ARCHIVE");

  let combined = binary.chain(interface);

  let token = vino_wascap::decode_token(jwt)?;
  let hash = vino_wascap::hash_bytes(combined)?;
  trace!("OCI:ARCHIVE:[Hash={}][Token={:?}]", hash, token);

  vino_wascap::assert_valid_jwt(&token, &hash)?;
  Ok(())
}

/// Validates an archive's contents by decoding the contained JWT and validating its hash against the binary and interface.
pub fn validate_provider_dir(dir: &Path) -> Result<(), ParError> {
  let jwt = std::fs::read(dir.join(JWT_PATH)).map_err(|_| ParError::MissingJwt)?;
  let interface = std::fs::File::open(dir.join(INTERFACE_PATH)).map_err(|_| ParError::MissingJwt)?;
  let bin = std::fs::File::open(dir.join(BIN_PATH)).map_err(|_| ParError::MissingJwt)?;
  validate_provider(bin, interface, jwt)
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use tar::Archive;

  use super::*;

  #[test_logger::test]
  fn test_archive_validation() -> Result<()> {
    let signature = ProviderSignature::default();
    let bin_bytes = b"0123456".to_vec();
    let claims = ClaimsOptions::default();
    let subject_kp = KeyPair::new_module();
    let issuer_kp = KeyPair::new_user();
    let archive_bytes = make_archive(&*bin_bytes, &signature, claims, &subject_kp, &issuer_kp)?;
    let mut archive = Archive::new(&*archive_bytes);
    let tmpdir = std::env::temp_dir().join("vinotest");
    archive.unpack(&tmpdir)?;

    validate_provider_dir(&tmpdir)?;

    Ok(())
  }
}
