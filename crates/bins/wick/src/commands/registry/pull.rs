use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use tracing::Instrument;
use wick_oci_utils::OciOptions;

use crate::options::get_auth_for_scope;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryPullCommand {
  /// OCI reference to pull.
  #[clap(action)]
  pub(crate) reference: String,

  /// Directory to store the pulled artifacts.
  #[clap(action)]
  pub(crate) output: Option<PathBuf>,

  /// Force overwriting of files.
  #[clap(short = 'f', long = "force", action)]
  pub(crate) force: bool,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

pub(crate) async fn pull(reference: String, oci_opts: OciOptions) -> Result<wick_package::WickPackage, anyhow::Error> {
  let pull_result = match wick_package::WickPackage::pull(&reference, &oci_opts).await {
    Ok(pull_result) => pull_result,
    Err(e) => {
      if let wick_package::Error::Oci(wick_oci_utils::error::OciError::WouldOverwrite(files)) = &e {
        warn!("Pulling {} will overwrite the following files", &reference);
        for file in files {
          warn!("{}", file.display());
        }
        error!("Refusing to overwrite files, pass --force to ignore.");
        return Err(anyhow!("Pull failed"));
      }
      error!("Failed to pull {}: {}", &reference, e);
      return Err(anyhow!("Pull failed"));
    }
  };
  Ok(pull_result)
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: RegistryPullCommand,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<()> {
  let configured_creds = settings
    .credentials
    .iter()
    .find(|c| opts.reference.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci_opts.username.as_deref(),
    opts.oci_opts.password.as_deref(),
  );

  let oci_opts = wick_oci_utils::OciOptions::default()
    .allow_insecure(opts.oci_opts.insecure_registries)
    .allow_latest(true)
    .username(username)
    .password(password)
    .overwrite(opts.force)
    .base_dir(opts.output);

  span.in_scope(|| debug!(options=?oci_opts, reference= opts.reference, "pulling reference"));

  let pull_result = pull(opts.reference, oci_opts).instrument(span.clone()).await;

  for file in pull_result?.list_files() {
    info!("Pulled file: {}", file.path().display());
  }
  Ok(())
}
