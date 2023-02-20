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
  clippy::if_then_some_else_none,
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
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs, clippy::expect_used)] // TODO docs

use wasmflow_manifest::WasmflowManifest;
use wasmflow_sdk::v1::transport::{TransportMap, TransportStream};

use crate::HostBuilder;

pub async fn run(
  manifest: WasmflowManifest,
  schematic: &str,
  data: TransportMap,
  seed: Option<u64>,
) -> crate::Result<TransportStream> {
  let host_builder = HostBuilder::from_definition(manifest);

  let mut host = host_builder.build();

  debug!("starting host");

  host.start(seed).await?;

  info!("manifest applied");

  let raw_result = host.request(schematic, data, None).await?;

  Ok(raw_result)
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use anyhow::Result;
  use wasmflow_sdk::v1::transport::TransportWrapper;

  use super::*;

  #[tokio::test]
  async fn runs_log_config() -> Result<()> {
    let host_def = WasmflowManifest::load_from_file(&PathBuf::from("./manifests/logger.wafl"))?;
    let input = vec![("input", "test-input")].into();

    let mut result = run(host_def, "logger", input, Some(0)).await?;
    let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
    let output: String = messages.remove(0).payload.deserialize()?;

    assert_eq!(output, "test-input");
    Ok(())
  }
}
