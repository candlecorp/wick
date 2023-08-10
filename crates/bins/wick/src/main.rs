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
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here

use std::path::PathBuf;
use std::thread::sleep;

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
mod wick_host;

pub(crate) use options::LoggingOptions;
use structured_output::StructuredOutput;
use tracing::Span;

use self::commands::*;
use self::options::{apply_log_settings, GlobalOptions};

static BIN_NAME: &str = "wick";
static BIN_DESC: &str = "wick runtime executable";

#[cfg(feature = "mem-profiler")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
  #[cfg(feature = "mem-profiler")]
  let _profiler = dhat::Profiler::new_heap();

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .thread_name("wick")
    .worker_threads(10)
    .build()
    .unwrap();
  let result = runtime.block_on(async_start());
  runtime.shutdown_background();

  let code = match result {
    Ok((options, output)) => {
      let (json, code) = if let Some(success) = output.json.as_object().unwrap().get("success") {
        if success.as_bool().unwrap() {
          (output.json.clone(), 0)
        } else {
          (output.json.clone(), 1)
        }
      } else {
        (serde_json::json!({"success":true,"output":&output.json}), 0)
      };

      if options.json {
        println!("{}", serde_json::to_string(&json).unwrap());
      } else {
        let output = output.to_string();
        if !output.is_empty() {
          println!("{}", output);
        }
      }
      code
    }
    Err((options, e)) => {
      let json = serde_json::json!({"error": e.to_string(),"success":false});

      if options.json {
        println!("{}", serde_json::to_string(&json).unwrap());
      } else {
        // Separate the error from the rest of the output with a few blank lines.
        for _ in 0..8 {
          println!();
          sleep(std::time::Duration::from_millis(20));
        }

        println!("\n{} exited with error: {}", BIN_NAME, e);
        println!("Run with --info, --debug, or --trace for more information.");
      }
      1
    }
  };

  #[cfg(feature = "mem-profiler")]
  drop(_profiler);

  std::process::exit(code);
}

async fn async_start() -> Result<(GlobalOptions, StructuredOutput), (GlobalOptions, anyhow::Error)> {
  #[cfg(debug_assertions)]
  panic::setup(human_panic::PanicStyle::Debug);
  #[cfg(not(debug_assertions))]
  panic::setup(human_panic::PanicStyle::Human);

  let mut args = std::env::args().collect::<Vec<_>>();

  let matches = <Cli as clap::CommandFactory>::command().try_get_matches_from(&args);
  if let Err(e) = matches {
    // If we have an invalid subcommand...
    if matches!(e.kind(), clap::error::ErrorKind::InvalidSubcommand) {
      // If we have an invalid subcommand and the subcommand is a filename that exists, make the default subcommand `wick run`
      if args.get(1).map_or_else(|| false, |arg| PathBuf::from(arg).exists()) {
        args.insert(1, "run".to_owned());
      }
    }
  }

  let mut cli = Cli::parse_from(args);
  let options = cli.output.clone();

  // Do a preliminary logger init to catch any logs from the local settings.
  // let log_options: wick_logger::LoggingOptions = cli.logging.name(BIN_NAME).into();
  // let settings = wick_logger::with_default(&log_options, Box::new(wick_settings::Settings::new));
  let settings = wick_settings::Settings::new();
  apply_log_settings(&settings, &mut cli.logging);

  let mut logger_opts: wick_logger::LoggingOptions = cli.logging.name(BIN_NAME).into();
  logger_opts.global = true;

  // Initialize the global logger
  let mut logger = wick_logger::init(&logger_opts);

  let span = info_span!(target:"cli","cli");

  let res = async_main(span.clone(), cli, settings).await;

  let res = span.in_scope(|| match res {
    Ok(output) => {
      debug!("Done");
      Ok((options, output))
    }
    Err(e) => {
      error!("Error: {}", e);
      Err((options, anyhow!("{}", e)))
    }
  });

  logger.flush();

  res
}

async fn async_main(span: Span, cli: Cli, settings: wick_settings::Settings) -> Result<StructuredOutput> {
  span.in_scope(|| trace!(cli_options=?cli, settings=?settings,"starting cli"));

  match cli.command {
    CliCommand::Serve(cmd) => commands::serve::handle(cmd, settings, span).await,
    CliCommand::List(cmd) => commands::list::handle(cmd, settings, span).await,
    CliCommand::Run(cmd) => commands::run::handle(cmd, settings, span).await,
    CliCommand::Invoke(cmd) => commands::invoke::handle(cmd, settings, span).await,
    CliCommand::Test(cmd) => commands::test::handle(cmd, settings, span).await,
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
      commands::registry::SubCommands::Manifest(cmd) => commands::registry::manifest::handle(cmd, settings, span).await,
    },
    CliCommand::Rpc(cmd) => match cmd {
      commands::rpc::SubCommands::Invoke(cmd) => commands::rpc::invoke::handle(cmd, settings, span).await,
      commands::rpc::SubCommands::List(cmd) => commands::rpc::list::handle(cmd, settings, span).await,
      commands::rpc::SubCommands::Stats(cmd) => commands::rpc::stats::handle(cmd, settings, span).await,
    },
    CliCommand::Query(cmd) => commands::query::handle(cmd, settings, span).await,
    CliCommand::Install(cmd) => commands::install::handle(cmd, settings, span).await,
    CliCommand::New(cmd) => match cmd {
      new::SubCommands::Component(cmd) => match cmd {
        new::component::SubCommands::Http(cmd) => new::component::http::handle(cmd, settings, span).await,
        new::component::SubCommands::Composite(cmd) => new::component::composite::handle(cmd, settings, span).await,
        new::component::SubCommands::Sql(cmd) => new::component::sql::handle(cmd, settings, span).await,
        new::component::SubCommands::WasmRS(cmd) => new::component::wasmrs::handle(cmd, settings, span).await,
      },
      new::SubCommands::App(cmd) => commands::new::app::handle(cmd, settings, span).await,
    },
    CliCommand::Show(cmd) => match cmd {
      show::SubCommands::Env(cmd) => commands::show::env::handle(cmd, settings, span).await,
    },
    CliCommand::Config(cmd) => match cmd {
      config::SubCommands::Dot(cmd) => commands::config::dot::handle(cmd, settings, span).await,
      config::SubCommands::Expand(cmd) => commands::config::expand::handle(cmd, settings, span).await,
    },
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }
}
