//! WasCap implementation for Wasmflow components

// !!START_LINTS
// Wasmflow lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow()]

/// The module that contains the component claims definition.
mod component;

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

pub use component::CollectionClaims;
use data_encoding::HEXUPPER;
use parity_wasm::elements::{CustomSection, Module, Serialize};
use parity_wasm::{deserialize_buffer, serialize};
use ring::digest::{Context, Digest, SHA256};
pub use wascap;
pub use wascap::jwt::Token;
pub use wascap::prelude::{validate_token, Claims, Invocation, KeyPair};
use wascap::wasm::days_from_now_to_jwt_time;
use wasmflow_sdk::v1::types::CollectionSignature;

/// The crate's error module.
pub mod error;
pub(crate) type Result<T> = std::result::Result<T, error::ClaimsError>;
pub use error::ClaimsError as Error;

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

/// Extract the claims embedded in a [Token].
pub fn extract_claims(contents: impl AsRef<[u8]>) -> Result<Option<Token<CollectionClaims>>> {
  let module: Module = deserialize_buffer(contents.as_ref())?;
  let sections: Vec<&CustomSection> = module.custom_sections().filter(|sect| sect.name() == "jwt").collect();

  if sections.is_empty() {
    Ok(None)
  } else {
    let token = decode_token(sections[0].payload().to_vec())?;
    let hash = compute_hash_without_jwt(module)?;
    assert_valid_jwt(&token, &hash)?;
    Ok(Some(token))
  }
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
  let mut module: Module = deserialize_buffer(orig_bytecode)?;
  module.clear_custom_section("jwt");
  let cleanbytes = serialize(module)?;
  let jwt = make_jwt(&*cleanbytes, claims, kp)?;

  let mut m: Module = deserialize_buffer(orig_bytecode)?;
  m.set_custom_section("jwt", jwt);
  let mut buf = Vec::new();
  m.serialize(&mut buf)?;

  Ok(buf)
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
  interface: CollectionSignature,
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
  interface: CollectionSignature,
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

fn compute_hash_without_jwt(module: Module) -> Result<String> {
  let mut refmod = module;
  refmod.clear_custom_section("jwt");
  let modbytes = serialize(refmod)?;
  let hash = hash_bytes(&*modbytes)?;

  Ok(hash)
}
