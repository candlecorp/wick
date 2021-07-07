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
    // missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow()]

pub mod commands;
pub mod error;
// pub(crate) mod logger;
pub mod utils;

use std::collections::HashMap;

use error::VinoError;
use serde_json::json;
use serde_json::Value::String as JsonString;
use vino_codec::messagepack::deserialize;
use vino_host::{
  HostBuilder,
  HostDefinition,
};
use vino_transport::MessageTransport;

pub type Result<T> = std::result::Result<T, VinoError>;
pub type Error = VinoError;

#[macro_use]
extern crate log;

pub type JsonMap = HashMap<String, serde_json::value::Value>;

pub async fn run(manifest: HostDefinition, data: JsonMap) -> Result<serde_json::Value> {
  let host_builder = HostBuilder::new();

  let mut host = host_builder.build();

  debug!("Starting host");

  host.start().await?;

  host.start_network(manifest.network).await?;

  info!("Manifest applied");

  let raw_result = host.request(&manifest.default_schematic, data).await?;

  debug!("Raw result: {:?}", raw_result);

  let result: serde_json::Value = raw_result
    .iter()
    .map(|(k, payload)| {
      (
        k.to_string(),
        match payload {
          MessageTransport::MessagePack(bytes) => deserialize(bytes).unwrap_or_else(|e| {
            JsonString(format!(
              "Error deserializing output payload: {}",
              e.to_string(),
            ))
          }),
          MessageTransport::Exception(e) => json!({ "exception": e }),
          MessageTransport::Error(e) => json!({ "error": e }),
          _ => json!({ "error": "Internal error, invalid format" }),
        },
      )
    })
    .collect();

  host.stop().await;

  Ok(result)
}

#[cfg(test)]
mod tests {

  use std::path::PathBuf;

  use maplit::hashmap;

  #[actix_rt::test]
  async fn runs_log_config() -> crate::Result<()> {
    let host_def =
      vino_host::HostDefinition::load_from_file(&PathBuf::from("./manifests/log.vino"))?;
    let input = hashmap! {
      "schem_input".into() => "test-input".into()
    };

    let result = crate::run(host_def, input).await?;
    assert_eq!(result.get("schem_output").unwrap(), "test-input");
    Ok(())
  }
}
