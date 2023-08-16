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
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let fetch_options: wick_oci_utils::OciOptions = opts.oci.clone().into();

  let with_config = parse_config_string(opts.component.with.as_deref())?;

  let mut manifest = WickConfiguration::fetch(&opts.component.path, fetch_options).await?;
  manifest.set_root_config(with_config);
  let manifest = manifest.finish()?.try_component_config()?;

  let config = merge_config(&manifest, &opts.oci, Some(opts.cli));

  let mut host = ComponentHostBuilder::default().manifest(config).span(span).build()?;

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
