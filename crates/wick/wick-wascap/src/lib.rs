//! WasCap implementation for Wick components

// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::await_holding_lock,
  clippy::borrow_as_ptr,
  clippy::branches_sharing_code,
  clippy::cast_lossless,
  clippy::clippy::collection_is_never_read,
  clippy::cloned_instead_of_copied,
  clippy::cognitive_complexity,
  clippy::create_dir,
  clippy::deref_by_slicing,
  clippy::derivable_impls,
  clippy::derive_partial_eq_without_eq,
  clippy::equatable_if_let,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::expl_impl_clone_on_copy,
  clippy::explicit_deref_methods,
  clippy::explicit_into_iter_loop,
  clippy::explicit_iter_loop,
  clippy::filetype_is_file,
  clippy::flat_map_option,
  clippy::format_push_string,
  clippy::fn_params_excessive_bools,
  clippy::future_not_send,
  clippy::get_unwrap,
  clippy::implicit_clone,
  clippy::if_then_some_else_none,
  clippy::impl_trait_in_params,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::inherent_to_string,
  clippy::iter_not_returning_iterator,
  clippy::large_types_passed_by_value,
  clippy::large_include_file,
  clippy::let_and_return,
  clippy::manual_assert,
  clippy::manual_ok_or,
  clippy::manual_split_once,
  clippy::manual_let_else,
  clippy::manual_string_new,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::missing_enforced_import_renames,
  clippy::missing_assert_message,
  clippy::missing_const_for_fn,
  clippy::must_use_candidate,
  clippy::mut_mut,
  clippy::needless_for_each,
  clippy::needless_option_as_deref,
  clippy::needless_pass_by_value,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::non_send_fields_in_send_ty,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::option_option,
  clippy::rc_mutex,
  clippy::redundant_else,
  clippy::same_name_method,
  clippy::semicolon_if_nothing_returned,
  clippy::str_to_string,
  clippy::string_to_string,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::trivial_regex,
  clippy::try_err,
  clippy::unnested_or_patterns,
  clippy::unused_async,
  clippy::unwrap_or_else_default,
  clippy::useless_let_if_seq,
  bad_style,
  clashing_extern_declarations,
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
#![warn(clippy::exhaustive_enums)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow()]
#![recursion_limit = "512"]

mod claims;
mod component;
mod error;
mod parser;
mod v0;
mod v1;

use ::base64::engine::general_purpose::URL_SAFE_NO_PAD as base64;
pub use claims::{extract_claims, sign_buffer_with_claims, validate_token, Claims, ClaimsOptions, Token};
pub use component::WickComponent;
pub use error::Error;

#[cfg(test)]
mod test {
  use anyhow::Result;
  use nkeys::KeyPair;
  use wick_interface_types::ComponentSignature;
  static MODULE_BYTES: &[u8] = include_bytes!("../test/test_wasi_component.wasm");

  use super::*;

  #[test]
  fn test_sign() -> Result<()> {
    let subject = KeyPair::new_service();
    let account = KeyPair::new_account();
    let signed = sign_buffer_with_claims(
      MODULE_BYTES,
      ComponentSignature::new_named("TEST"),
      &subject,
      &account,
      &ClaimsOptions::v0(None, None, None, None),
    )?;
    let claims = extract_claims(signed)?.unwrap();
    assert_eq!(claims.claims.name(), "TEST");

    Ok(())
  }

  #[rstest::rstest]
  #[case(
    "./test/1.v0.signed.wasm",
    "D3DFCF7F12B01A22025B2341871A46B5A4EE71387B32EE857EDBE69F2D1E1299",
    "90E5D03AF45BAE5EFC5841C196A2774BEB783E4E041E1D6D1421073765D47E50"
  )]
  #[case(
    "./test/2.v0.signed.wasm",
    "2535F3568A2E0798AA376A6F836A65C81F1A258156F9E98E94B33A0E42EFC2C2",
    "846CBC6E9D35321E0A81D150B1CCA2816EAD9E53DAF0AA12BD2FB44E19E7605C"
  )]
  #[case(
    "./test/3.v0.signed.wasm",
    "8CF411C08AEEF40150E70E0210A4C5A67559871FDB43351664A42DC6F94B8DC5",
    "7A68971E61256D7B76FA580B2E17B173B943B1B737E65C5EB6AECA6D37312EEE"
  )]
  #[case(
    "./test/4.v0.signed.wasm",
    "7E215B19354779A37A5C01740D8D129536C38E1A2659A916F440418129924A11",
    "8DD28458BE618E260A70390FAEB5E74160823F979A32F6167F3C3D3D1C2C08BB"
  )]
  fn test_re_sign(#[case] file: &str, #[case] old_hash: &str, #[case] new_hash: &str) -> Result<()> {
    let subject = KeyPair::new_service();
    let account = KeyPair::new_account();

    let bytes = std::fs::read(file)?;
    let token = extract_claims(&bytes)?.unwrap();

    assert_eq!(token.claims.metadata.as_ref().unwrap().module_hash, old_hash);

    validate_token::<WickComponent>(&token.jwt)?;

    let signed = sign_buffer_with_claims(
      &bytes,
      ComponentSignature::new_named("TEST"),
      &subject,
      &account,
      &ClaimsOptions::v1(None, None, None),
    )?;

    wasmparser::validate(&signed)?;

    let token = extract_claims(&signed)?.unwrap();

    assert_eq!(token.claims.metadata.as_ref().unwrap().module_hash, new_hash);

    validate_token::<WickComponent>(&token.jwt)?;

    Ok(())
  }
}
