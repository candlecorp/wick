use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryPullCommand {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

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

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: RegistryPullCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  let oci_opts = wick_oci_utils::OciOptions::default()
    .allow_insecure(opts.oci_opts.insecure_registries)
    .allow_latest(true)
    .username(opts.oci_opts.username)
    .password(opts.oci_opts.password)
    .overwrite(opts.force)
    .base_dir(opts.output);

  debug!(options=?oci_opts, reference= opts.reference, "pulling reference");

  let pull_result = match wick_package::WickPackage::pull(&opts.reference, &oci_opts).await {
    Ok(pull_result) => pull_result,
    Err(e) => {
      if let wick_package::Error::Oci(wick_oci_utils::error::OciError::WouldOverwrite(files)) = &e {
        info!("Pulling {} will overwrite the following files", opts.reference);
        for file in files {
          info!("{}", file.display());
        }
        error!("Refusing to overwrite files, pass --force to ignore.");
        return Err(anyhow!("Pull failed"));
      }
      error!("Failed to pull {}: {}", opts.reference, e);
      return Err(e.into());
    }
  };
  for file in pull_result.list_files() {
    info!("Pulled file: {}", file.path().display());
  }
  Ok(())
}
