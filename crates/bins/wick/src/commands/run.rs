use std::collections::HashMap;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use tracing::Instrument;
use wick_config::WickConfiguration;
use wick_host::AppHostBuilder;

use crate::utils::{fetch_wick_config, parse_config_string, reconcile_fetch_options};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,

  /// Use the given lockdown configuration to restrict the app's behavior.
  #[clap(long = "lockdown", short = 'l', action)]
  lockdown: Option<String>,

  /// Don't run the application, just fetch and validate the configuration.
  #[clap(long = "dryrun", action)]
  dryrun: bool,

  /// Arguments to pass as inputs to a CLI trigger in the application.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| trace!(args = ?opts.args, "rest args"));
  let runtime_config = parse_config_string(opts.component.with.as_deref())?;
  let options = reconcile_fetch_options(&opts.component.path, &settings, opts.oci, None);

  let config = if let Some(lockdown) = opts.lockdown {
    let env: HashMap<String, String> = std::env::vars().collect();

    let mut lockdown_config = WickConfiguration::fetch(&lockdown, options.clone())
      .instrument(span.clone())
      .await?;
    lockdown_config.set_env(env.clone());
    let lockdown_config = lockdown_config.finish()?.try_lockdown_config()?;

    let tree = WickConfiguration::fetch_tree(&opts.component.path, runtime_config, env, options.clone()).await?;
    let mut flattened = tree.flatten();
    wick_config::lockdown::assert_restrictions(&flattened, &lockdown_config)?;

    flattened.remove(0).as_config().unwrap()
  } else {
    fetch_wick_config(&opts.component.path, options.clone(), runtime_config, span.clone()).await?
  };

  let mut app_config = config.try_app_config()?;

  app_config.set_options(options);

  let mut host = AppHostBuilder::default()
    .manifest(app_config.clone())
    .span(span.clone())
    .build()?;

  if !opts.dryrun {
    host.start(opts.component.seed)?;
    span.in_scope(|| debug!("Waiting on triggers to finish..."));

    host.wait_for_done().instrument(span.clone()).await?;
  } else {
    info!("application valid but not started because --dryrun set");
  }

  Ok(StructuredOutput::new("", json!({})))
}
