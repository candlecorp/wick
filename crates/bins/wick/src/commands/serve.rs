use anyhow::Result;
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
  pub(crate) oci: crate::oci::Options,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,
}

pub(crate) async fn handle(opts: ServeCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  let fetch_options = wick_config::config::FetchOptions::new()
    .allow_latest(opts.oci.allow_latest)
    .allow_insecure(&opts.oci.insecure_registries);

  let manifest = WickConfiguration::fetch_all(&opts.location, fetch_options)
    .await?
    .try_component_config()?;

  let config = merge_config(&manifest, &opts.oci, Some(opts.cli));

  let mut host = ComponentHostBuilder::default().manifest(config).span(span).build()?;

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
