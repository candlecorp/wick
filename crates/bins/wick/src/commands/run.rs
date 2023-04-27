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

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = wick_logger::init(&opts.logging.name(crate::BIN_NAME));

  debug!(args = ?opts.args, "rest args");

  let wick_config = WickConfiguration::load_from_file(&opts.path).await;

  let app_config = match wick_config {
    Ok(app_config) => app_config.try_app_config().unwrap(),
    Err(_) => {
      let oci_opts = wick_oci_utils::OciOptions::default()
        .allow_insecure(opts.oci_opts.insecure_oci_registries)
        .allow_latest(true)
        .username(opts.oci_opts.username)
        .password(opts.oci_opts.password)
        .overwrite(true);
      let app_pull = crate::commands::registry::pull::pull(opts.path, oci_opts)
        .await
        .unwrap();

      WickConfiguration::load_from_file(app_pull.path())
        .await?
        .try_app_config()
        .unwrap()
    }
  };

  let mut host = AppHostBuilder::from_definition(app_config.clone()).build();
  host.start(opts.seed)?;
  debug!("Waiting on triggers to finish or interrupt...");
  host.wait_for_done().await?;

  Ok(())
}
