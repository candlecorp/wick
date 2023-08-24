use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_host::Host;

use crate::utils::parse_config_string;
use crate::wick_host::build_host;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| debug!("expand config"));
  let root_config = parse_config_string(opts.component.with.as_deref())?;
  let host = build_host(&opts.component.path, opts.oci, root_config, settings, None, None, span).await?;

  let config = host.get_active_config();
  let signature = host.get_signature(None, None)?;

  let config_yaml = config.clone().into_v1_yaml()?;
  let config_json = serde_json::to_value(&config)?;

  let json = json!({"signature": signature, "config": config_json});
  Ok(StructuredOutput::new(config_yaml, json))
}
