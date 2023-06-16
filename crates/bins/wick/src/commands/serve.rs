use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;

use crate::utils::{merge_config, parse_config_string};

#[derive(Debug, Clone, Args)]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) cli: DefaultCliOptions,

  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,

  /// Pass configuration necessary to instantiate the component (JSON).
  #[clap(long = "with", short = 'w', action)]
  with: Option<String>,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let fetch_options: wick_oci_utils::OciOptions = opts.oci.clone().into();

  let manifest = WickConfiguration::fetch_all(&opts.location, fetch_options)
    .await?
    .try_component_config()?;

  let config = merge_config(&manifest, &opts.oci, Some(opts.cli));
  let component_config = parse_config_string(opts.with.as_deref())?;

  let mut host = ComponentHostBuilder::default()
    .manifest(config)
    .config(component_config)
    .span(span)
    .build()?;

  host.start(None).await?;
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
  Ok(StructuredOutput::new("", json!({})))
}
