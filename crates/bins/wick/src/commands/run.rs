use anyhow::Result;
use clap::Args;
use tracing::Instrument;
use wick_config::WickConfiguration;
use wick_host::AppHostBuilder;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  path: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a CLI trigger in the application.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(opts: RunCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  span.in_scope(|| trace!(args = ?opts.args, "rest args"));
  let app_config = WickConfiguration::fetch_all(&opts.path, opts.oci.into())
    .instrument(span.clone())
    .await?
    .try_app_config()?;

  let mut host = AppHostBuilder::default()
    .manifest(app_config.clone())
    .span(span.clone())
    .build()?;

  host.start(opts.seed)?;
  span.in_scope(|| debug!("Waiting on triggers to finish or interrupt..."));

  host.wait_for_done().instrument(span.clone()).await?;

  Ok(())
}
