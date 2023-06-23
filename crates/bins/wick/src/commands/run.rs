use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use tracing::Instrument;
use wick_config::WickConfiguration;
use wick_host::AppHostBuilder;

use crate::options::get_auth_for_scope;
use crate::utils::parse_config_string;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  path: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Pass configuration necessary to instantiate the application or its resources (JSON).
  #[clap(long = "with", short = 'w', action)]
  with: Option<String>,

  /// Arguments to pass as inputs to a CLI trigger in the application.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let xdg = wick_xdg::Settings::new();
  span.in_scope(|| trace!(args = ?opts.args, "rest args"));

  let configured_creds = settings.credentials.iter().find(|c| opts.path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci.username.as_deref(),
    opts.oci.password.as_deref(),
  );

  let mut fetch_opts: wick_oci_utils::OciOptions = opts.oci.clone().into();
  fetch_opts.set_username(username).set_password(password);

  let path = PathBuf::from(&opts.path);

  if !path.exists() {
    fetch_opts.set_cache_dir(xdg.global().cache().clone());
  } else {
    let mut path_dir = path.clone();
    path_dir.pop();
    fetch_opts.set_cache_dir(path_dir.join(xdg.local().cache()));
  };

  let mut builder = WickConfiguration::fetch_all(&opts.path, fetch_opts.clone())
    .instrument(span.clone())
    .await?;

  let with_config = parse_config_string(opts.with.as_deref())?;

  builder
    .set_root_config(with_config)
    .set_env(Some(std::env::vars().collect()));
  let mut app_config = builder.finish()?.try_app_config()?;

  app_config.set_options(fetch_opts);

  let mut host = AppHostBuilder::default()
    .manifest(app_config.clone())
    .span(span.clone())
    .build()?;

  host.start(opts.seed)?;
  span.in_scope(|| debug!("Waiting on triggers to finish..."));

  host.wait_for_done().instrument(span.clone()).await?;

  Ok(StructuredOutput::new("", json!({})))
}
