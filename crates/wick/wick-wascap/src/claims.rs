use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use data_encoding::HEXUPPER;
// use parity_wasm::elements::{CustomSection, Module, Serialize};
// use parity_wasm::{deserialize_buffer, serialize};
use ring::digest::{Context, Digest, SHA256};
use wascap::jwt::Token;
use wascap::prelude::{Claims, KeyPair};
use wascap::wasm::days_from_now_to_jwt_time;
use wick_interface_types::ComponentSignature;

use crate::component::CollectionClaims;
use crate::error;
use crate::parser::strip_custom_section;

type Result<T> = std::result::Result<T, error::ClaimsError>;

/// A common struct to group related options together.
#[derive(Debug, Default, Clone)]
pub struct ClaimsOptions {
  /// The revision of the claims target.
  pub revision: Option<u32>,
  /// The version of the claims target.
  pub version: Option<String>,
  /// When the target expires.
  pub expires_in_days: Option<u64>,
  /// When the target becomes valid.
  pub not_before_days: Option<u64>,
}

const SECTION_JWT: &str = "jwt"; // Versions of wascap prior to 0.9 used this section

/// Extract the claims embedded in a WebAssembly module.
pub fn extract_claims(contents: impl AsRef<[u8]>) -> Result<Option<Token<CollectionClaims>>> {
  let stripped_bytes = strip_custom_section(contents.as_ref())?;
  let target_hash = hash_bytes(&*stripped_bytes)?;
  println!("target_hash: {}", target_hash);
  let parser = wasmparser::Parser::new(0);
  for payload in parser.parse_all(contents.as_ref()) {
    let payload = payload?;
    use wasmparser::Payload::*;
    match payload {
      CustomSection(c) if (c.name() == SECTION_JWT) => {
        let jwt = String::from_utf8(c.data().to_vec())?;
        let claims: Claims<CollectionClaims> = Claims::decode(&jwt)?;
        if let Some(ref meta) = claims.metadata {
          println!("extracted_hash: {}", meta.module_hash);
          if meta.module_hash != target_hash {
            // return Err(error::ClaimsError::InvalidModuleHash);
          }
          return Ok(Some(Token { jwt, claims }));
        }
        return Err(error::ClaimsError::InvalidModuleFormat);
      }
      _ => {}
    }
  }
  Ok(None)
}

/// Validate a JWT's hash matches the passed hash.
pub fn assert_valid_jwt(token: &Token<CollectionClaims>, hash: &str) -> Result<()> {
  let valid_hash = token
    .claims
    .metadata
    .as_ref()
    .map_or(false, |meta| meta.module_hash == hash);

  if valid_hash {
    Ok(())
  } else {
    Err(error::ClaimsError::InvalidModuleHash)
  }
}

/// Decode a JWT and its claims.
pub fn decode_token(jwt_bytes: Vec<u8>) -> Result<Token<CollectionClaims>> {
  let jwt = String::from_utf8(jwt_bytes)?;
  tracing::trace!(%jwt, "jwt");
  let claims: Claims<CollectionClaims> = Claims::decode(&jwt)?;
  Ok(Token { jwt, claims })
}

/// This function will embed a set of claims inside the bytecode of a WebAssembly module. The claims.
/// are converted into a JWT and signed using the provided `KeyPair`.
pub fn embed_claims(orig_bytecode: &[u8], claims: &Claims<CollectionClaims>, kp: &KeyPair) -> Result<Vec<u8>> {
  let mut bytes = orig_bytecode.to_vec();
  bytes = strip_custom_section(&bytes)?;

  let hash = hash_bytes(&*bytes)?;
  let mut claims = (*claims).clone();
  let meta = claims.metadata.map(|md| CollectionClaims {
    module_hash: hash,
    ..md
  });
  claims.metadata = meta;

  let encoded = claims.encode(kp)?;
  let encvec = encoded.as_bytes().to_vec();
  wasm_gen::write_custom_section(&mut bytes, SECTION_JWT, &encvec);

  Ok(bytes)
}

#[derive(Debug)]
struct ClaimsJwt {
  data: Vec<u8>,
}

/// Create a JWT claims with a hash of the buffer embedded.
pub fn make_jwt<R: Read>(buffer: R, claims: &Claims<CollectionClaims>, kp: &KeyPair) -> Result<Vec<u8>> {
  let module_hash = hash_bytes(buffer)?;
  let mut claims = (*claims).clone();
  let meta = claims.metadata.map(|md| CollectionClaims { module_hash, ..md });
  claims.metadata = meta;

  let encoded = claims.encode(kp)?;
  let encvec = encoded.as_bytes().to_vec();

  Ok(encvec)
}

/// Create a string safe hash from a list of bytes.
pub fn hash_bytes<R: Read>(buffer: R) -> Result<String> {
  let digest = sha256_digest(buffer)?;
  Ok(HEXUPPER.encode(digest.as_ref()))
}

/// Build collection claims from passed values
#[must_use]
pub fn build_collection_claims(
  interface: ComponentSignature,
  subject_kp: &KeyPair,
  issuer_kp: &KeyPair,
  options: ClaimsOptions,
) -> Claims<CollectionClaims> {
  Claims::<CollectionClaims> {
    expires: options.expires_in_days,
    id: nuid::next(),
    issued_at: since_the_epoch().as_secs(),
    issuer: issuer_kp.public_key(),
    subject: subject_kp.public_key(),
    not_before: days_from_now_to_jwt_time(options.not_before_days),
    metadata: Some(CollectionClaims {
      module_hash: "".to_owned(),
      tags: Some(Vec::new()),
      interface,
      rev: options.revision,
      ver: options.version,
    }),
  }
}

#[allow(clippy::too_many_arguments)]
/// Sign WebAssembly bytes with the passed claims.
pub fn sign_buffer_with_claims(
  buf: impl AsRef<[u8]>,
  interface: ComponentSignature,
  mod_kp: &KeyPair,
  acct_kp: &KeyPair,
  options: ClaimsOptions,
) -> Result<Vec<u8>> {
  let claims = build_collection_claims(interface, mod_kp, acct_kp, options);

  embed_claims(buf.as_ref(), &claims, acct_kp)
}

fn since_the_epoch() -> std::time::Duration {
  let start = SystemTime::now();
  start.duration_since(UNIX_EPOCH).unwrap()
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
  let mut context = Context::new(&SHA256);
  let mut buffer = [0; 1024];

  loop {
    let count = reader.read(&mut buffer)?;
    if count == 0 {
      break;
    }
    context.update(&buffer[..count]);
  }

  Ok(context.finish())
}
