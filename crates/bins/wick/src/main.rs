#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/104781277?s=96&v=4")]
#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
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
#![allow(clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here

use anyhow::Result;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
mod commands;
mod utils;
mod wasm;
use clap::Parser;
mod io;
mod keys;
mod oci;
mod options;
mod panic;

pub(crate) use options::LoggingOptions;

use self::commands::*;
use self::options::apply_log_settings;

static BIN_NAME: &str = "wick";
static BIN_DESC: &str = "wick runtime executable";

fn main() {
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .thread_name("wick")
    .worker_threads(10)
    .build()
    .unwrap();
  let result = runtime.block_on(async_start());
  runtime.shutdown_background();

  let code = match result {
    Ok(_) => 0,
    Err(e) => {
      eprintln!("\n{} exited with error: {}", BIN_NAME, e);
      eprintln!("Run with --info, --debug, or --trace for more information.");
      1
    }
  };

  std::process::exit(code);
}

async fn async_start() -> Result<()> {
  #[cfg(debug_assertions)]
  panic::setup(human_panic::PanicStyle::Debug);
  #[cfg(not(debug_assertions))]
  panic::setup(human_panic::PanicStyle::Human);

  let mut cli = Cli::parse();

  // Do a preliminary logger init to catch any logs from the local settings.
  let log_options: wick_logger::LoggingOptions = cli.logging.name(BIN_NAME).into();
  let settings = wick_logger::with_default(&log_options, Box::new(wick_settings::Settings::new));
  apply_log_settings(&settings, &mut cli.logging);

  let mut logger_opts: wick_logger::LoggingOptions = cli.logging.name(BIN_NAME).into();
  logger_opts.global = true;

  // Initialize the global logger
  let logger = wick_logger::init(&logger_opts);

  let res = tokio::spawn(async_main(cli, settings)).await?;
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  match &res {
    Ok(_) => {
      info!("Done");
    }
    Err(e) => {
      error!("Error: {}", e);
    }
  };
  logger.flush();
  res
}

async fn async_main(cli: Cli, settings: wick_settings::Settings) -> Result<()> {
  let span = trace_span!(target:"cli","cli");

  match cli.command {
    CliCommand::Serve(cmd) => commands::serve::handle(cmd, settings, span).await,
    CliCommand::List(cmd) => commands::list::handle(cmd, settings, span).await,
    CliCommand::Run(cmd) => commands::run::handle(cmd, settings, span).await,
    CliCommand::Invoke(cmd) => commands::invoke::handle(cmd, settings, span).await,
    CliCommand::Test(cmd) => commands::test::handle(cmd, settings, span).await,
    CliCommand::Init(cmd) => commands::init::handle(cmd, settings, span).await,
    CliCommand::Wasm(cmd) => match cmd {
      commands::wasm::SubCommands::Sign(cmd) => commands::wasm::sign::handle(cmd, settings, span).await,
      commands::wasm::SubCommands::Inspect(cmd) => commands::wasm::inspect::handle(cmd, settings, span).await,
    },
    CliCommand::Key(cmd) => match cmd {
      commands::key::SubCommands::Get(cmd) => commands::key::get::handle(cmd, settings, span).await,
      commands::key::SubCommands::Gen(cmd) => commands::key::gen::handle(cmd, settings, span).await,
      commands::key::SubCommands::List(cmd) => commands::key::list::handle(cmd, settings, span).await,
    },
    CliCommand::Registry(cmd) => match cmd {
      commands::registry::SubCommands::Push(cmd) => commands::registry::push::handle(cmd, settings, span).await,
      commands::registry::SubCommands::Pull(cmd) => commands::registry::pull::handle(cmd, settings, span).await,
      commands::registry::SubCommands::Login(cmd) => commands::registry::login::handle(cmd, settings, span).await,
    },
    CliCommand::Rpc(cmd) => match cmd {
      commands::rpc::SubCommands::Invoke(cmd) => commands::rpc::invoke::handle(cmd, settings, span).await,
      commands::rpc::SubCommands::List(cmd) => commands::rpc::list::handle(cmd, settings, span).await,
      commands::rpc::SubCommands::Stats(cmd) => commands::rpc::stats::handle(cmd, settings, span).await,
    },
    CliCommand::Query(cmd) => commands::query::handle(cmd, settings, span).await,
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn cli_tests() {
    trycmd::TestCases::new().case("tests/cmd/*.trycmd");
  }

  #[test]
  fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }
}
