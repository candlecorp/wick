use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_config::WickConfiguration;

use crate::utils::parse_config_string;
use crate::wick_host::build_component_host;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Path to composite component to load.
  #[clap(action)]
  pub(crate) path: String,

  /// Operation to render.
  #[clap(action)]
  pub(crate) operation: String,

  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// Pass configuration necessary to instantiate the component (JSON).
  #[clap(long = "with", short = 'w', action)]
  with: Option<String>,

  /// Pass configuration necessary to invoke the operation (JSON).
  #[clap(long = "op-with", action)]
  op_with: Option<String>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| debug!("Generate dotviz graph"));
  let root_config = parse_config_string(opts.with.as_deref())?;
  let host = build_component_host(&opts.path, opts.oci, root_config, settings, None, None, span).await?;

  let config = host.get_active_config()?;
  let config = WickConfiguration::Component(config.clone());
  let config_yaml = config.clone().into_v1_yaml()?;
  let config_json = serde_json::to_value(&config)?;
  let signature = host.get_signature()?;

  let json = json!({"signature": signature, "config": config_json});
  Ok(StructuredOutput::new(config_yaml, json))
}
