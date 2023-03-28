use anyhow::{Context, Result};
use clap::Args;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
pub(crate) struct ServeCommand {
  #[clap(flatten)]
  pub(crate) cli: DefaultCliOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,
}

pub(crate) async fn handle_command(opts: ServeCommand) -> Result<()> {
  let _guard = logger::init(&opts.cli.logging.name(crate::BIN_NAME));

  let bytes = wick_loader_utils::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  let manifest = WickConfiguration::load_from_bytes(&bytes, &Some(opts.location))?.try_component_config()?;

  let config = merge_config(&manifest, &opts.fetch, Some(opts.cli));

  let host_builder = ComponentHostBuilder::from_definition(config);

  let mut host = host_builder.build();

  host.start(Some(0)).await?;
  info!("Host started");
  #[allow(clippy::option_if_let_else)]
  match host.get_server_info() {
    Some(info) => {
      wick_component_cli::print_info(info);
    }
    None => {
      warn!("No server information available, did you intend to start a host without GRPC or a mesh connection?");
    }
  };
  info!("Waiting for Ctrl-C");
  let _ = tokio::signal::ctrl_c().await;
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok(())
}
