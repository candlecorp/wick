use anyhow::Result;
use clap::Args;
use wick_config::WickConfiguration;
use wick_host::AppHostBuilder;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

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

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  debug!(args = ?opts.args, "rest args");

  let app_config = WickConfiguration::load_from_file(&opts.path).await?.try_app_config()?;
  let mut host = AppHostBuilder::from_definition(app_config.clone()).build();
  host.start(opts.seed)?;
  host.wait_for_done().await?;

  Ok(())
}
