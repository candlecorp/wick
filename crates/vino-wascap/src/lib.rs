// !!START_LINTS
// Vino lints
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
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
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
    path_statements ,
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
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

pub mod component;

use std::io::Read;
use std::time::{
  SystemTime,
  UNIX_EPOCH,
};

pub use component::ComponentClaims;
use data_encoding::HEXUPPER;
use parity_wasm::elements::{
  CustomSection,
  Module,
  Serialize,
};
use parity_wasm::{
  deserialize_buffer,
  serialize,
};
use ring::digest::{
  Context,
  Digest,
  SHA256,
};
use vino_rpc::ProviderSignature;
pub use wascap::jwt::Token;
pub use wascap::prelude::{
  validate_token,
  Claims,
  Invocation,
  KeyPair,
};
use wascap::wasm::days_from_now_to_jwt_time;

pub mod error;
pub(crate) type Result<T> = std::result::Result<T, error::ClaimsError>;

pub fn extract_claims(contents: impl AsRef<[u8]>) -> Result<Option<Token<ComponentClaims>>> {
  let module: Module = deserialize_buffer(contents.as_ref())?;
  let sections: Vec<&CustomSection> = module
    .custom_sections()
    .filter(|sect| sect.name() == "jwt")
    .collect();

  if sections.is_empty() {
    Ok(None)
  } else {
    let jwt = String::from_utf8(sections[0].payload().to_vec())?;
    let claims: Claims<ComponentClaims> = Claims::decode(&jwt)?;
    let hash = compute_hash_without_jwt(module)?;

    if let Some(ref meta) = claims.metadata {
      if meta.module_hash != hash {
        Err(error::ClaimsError::InvalidModuleHash)
      } else {
        Ok(Some(Token { jwt, claims }))
      }
    } else {
      Err(error::ClaimsError::InvalidAlgorithm)
    }
  }
}

/// This function will embed a set of claims inside the bytecode of a WebAssembly module. The claims
/// are converted into a JWT and signed using the provided `KeyPair`.
/// According to the WebAssembly [custom section](https://webassembly.github.io/spec/core/appendix/custom.html)
/// specification, arbitary sets of bytes can be stored in a WebAssembly module without impacting
/// parsers or interpreters. Returns a vector of bytes representing the new WebAssembly module which can
/// be saved to a `.wasm` file
pub fn embed_claims(
  orig_bytecode: &[u8],
  claims: &Claims<ComponentClaims>,
  kp: &KeyPair,
) -> Result<Vec<u8>> {
  let mut module: Module = deserialize_buffer(orig_bytecode)?;
  module.clear_custom_section("jwt");
  let cleanbytes = serialize(module)?;

  let digest = sha256_digest(cleanbytes.as_slice())?;
  let mut claims = (*claims).clone();
  let meta = claims.metadata.map(|md| ComponentClaims {
    module_hash: HEXUPPER.encode(digest.as_ref()),
    ..md
  });
  claims.metadata = meta;

  let encoded = claims.encode(kp)?;
  let encvec = encoded.as_bytes().to_vec();
  let mut m: Module = deserialize_buffer(orig_bytecode)?;
  m.set_custom_section("jwt", encvec);
  let mut buf = Vec::new();
  m.serialize(&mut buf)?;

  Ok(buf)
}

#[allow(clippy::too_many_arguments)]
pub fn sign_buffer_with_claims(
  buf: impl AsRef<[u8]>,
  interface: ProviderSignature,
  mod_kp: KeyPair,
  acct_kp: KeyPair,
  expires_in_days: Option<u64>,
  not_before_days: Option<u64>,
  version: Option<String>,
  revision: Option<u32>,
) -> Result<Vec<u8>> {
  let claims = Claims::<ComponentClaims> {
    expires: expires_in_days,
    id: nuid::next(),
    issued_at: since_the_epoch().as_secs(),
    issuer: acct_kp.public_key(),
    subject: mod_kp.public_key(),
    not_before: days_from_now_to_jwt_time(not_before_days),
    metadata: Some(ComponentClaims {
      module_hash: "".to_owned(),
      tags: Some(Vec::new()),
      interface,
      rev: revision,
      ver: version,
    }),
  };

  embed_claims(buf.as_ref(), &claims, &acct_kp)
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

  let digest = sha256_digest(modbytes.as_slice())?;
  Ok(HEXUPPER.encode(digest.as_ref()))
}
