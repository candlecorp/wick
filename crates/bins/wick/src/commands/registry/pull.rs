use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use wick_oci_utils::OciOptions;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryPullCommand {
  #[clap(flatten)]
  pub(crate) logging: wick_logger::LoggingOptions,

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
pub(crate) async fn handle(opts: RegistryPullCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  let oci_opts = wick_oci_utils::OciOptions::default()
    .allow_insecure(opts.oci_opts.insecure_oci_registries)
    .allow_latest(true)
    .username(opts.oci_opts.username)
    .password(opts.oci_opts.password)
    .overwrite(opts.force)
    .base_dir(opts.output);

  debug!(options=?oci_opts, reference= opts.reference, "pulling reference");

  let pull_result = pull(opts.reference, oci_opts).await;

  for file in pull_result.unwrap().list_files() {
    info!("Pulled file: {}", file.path().display());
  }
  Ok(())
}
