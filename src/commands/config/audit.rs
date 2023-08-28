use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_config::audit::Audit;
use wick_config::WickConfiguration;

use crate::utils::{fetch_wick_tree, parse_config_string, reconcile_fetch_options};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,

  /// Turn the audit report into a lockdown configuration.
  #[clap(long = "lockdown", action)]
  lockdown: bool,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let runtime_config = parse_config_string(opts.component.with.as_deref())?;
  let options = reconcile_fetch_options(&opts.component.path, &settings, opts.oci, None);
  let config = fetch_wick_tree(&opts.component.path, options.clone(), runtime_config, span.clone()).await?;
  let flattened = config.flatten();
  let report = Audit::new_flattened(&flattened);

  if opts.lockdown {
    let config = WickConfiguration::Lockdown(report.into());
    let config_json = serde_json::to_value(&config)?;
    let config_yaml = config.into_v1_yaml()?;

    Ok(StructuredOutput::new(config_yaml, config_json))
  } else {
    let mut buffer = String::new();
    gen_report(&report, &mut buffer);
    let filtered = report.iter().filter(|a| !a.resources.is_empty()).collect::<Vec<_>>();
    let json = json!({"audit":filtered});

    Ok(StructuredOutput::new(buffer, json))
  }
}

fn gen_report(audit: &[Audit], buffer: &mut String) {
  for audit in audit {
    if audit.resources.is_empty() {
      continue;
    }
    let resource_lines = audit
      .resources
      .iter()
      .map(|r| format!("    {}", r))
      .collect::<Vec<_>>()
      .join("\n");
    let message = format!(
      "
name: {}
  resources:
{}",
      audit.name, resource_lines
    );
    buffer.push_str(&message);
  }
}
