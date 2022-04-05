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
#![allow(
  clippy::future_not_send,
  missing_docs, // TODO
  clippy::expect_used, // because of tokio::main
  clippy::semicolon_if_nothing_returned // because of tokio::main
)]

use clap::{AppSettings, Parser};

pub(crate) mod commands;
pub mod error;

pub(crate) type Result<T> = std::result::Result<T, error::VowError>;

#[macro_use]
extern crate tracing;

#[derive(Parser, Debug, Clone)]
#[clap(
     global_setting(
      AppSettings::DeriveDisplayOrder
     ),
     name = BIN_NAME, about = "Vino WebAssembly Wrapper")]
pub(crate) struct Cli {
  #[clap(subcommand)]
  pub(crate) command: commands::CliCommand,
}

static BIN_NAME: &str = "vow";

#[tokio::main]
async fn main() {
  let opts = Cli::parse();

  let res = run(opts).await;

  let result = match res {
    Ok(_) => 0,
    Err(e) => {
      debug!("Error: {:?}", e);
      eprintln!("{} exiting with error: {}", BIN_NAME, e);
      eprintln!("Run with --info, --debug, or --trace for more information.");
      1
    }
  };

  std::process::exit(result);
}

async fn run(opts: Cli) -> Result<()> {
  match opts.command {
    commands::CliCommand::Test(cmd) => commands::test_cmd::handle_command(cmd).await,
    commands::CliCommand::Run(cmd) => commands::run::handle_command(cmd).await,
    commands::CliCommand::Serve(cmd) => commands::serve::handle_command(cmd).await,
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn verify_options() {
    use clap::IntoApp;
    super::Cli::command().debug_assert();
  }

  #[test]
  fn cli_tests() {
    trycmd::TestCases::new().case("tests/cmd/*.trycmd");
  }
}
