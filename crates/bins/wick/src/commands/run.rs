use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use tracing::Instrument;
use wick_config::{FetchOptions, WickConfiguration};
use wick_host::AppHostBuilder;

use crate::options::get_auth_for_scope;

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

pub(crate) async fn handle(opts: RunCommand, settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  span.in_scope(|| trace!(args = ?opts.args, "rest args"));

  let configured_creds = settings.credentials.iter().find(|c| opts.path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci.username.as_deref(),
    opts.oci.password.as_deref(),
  );

  let mut fetch_opts = FetchOptions::default()
    .allow_insecure(opts.oci.insecure_registries)
    .allow_latest(true);
  if let Some(username) = username {
    fetch_opts = fetch_opts.oci_username(username);
  }
  if let Some(password) = password {
    fetch_opts = fetch_opts.oci_password(password);
  }

  if !PathBuf::from(&opts.path).exists() {
    fetch_opts = fetch_opts.artifact_dir(wick_xdg::Directories::GlobalCache.basedir()?);
  };

  let app_config = WickConfiguration::fetch_all(&opts.path, fetch_opts)
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
