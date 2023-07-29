use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  /// Username to use when pushing/pulling from an OCI registry
  #[clap(long, env = "OCI_USERNAME", action)]
  pub(crate) username: Option<String>,

  /// Password to use with username when pushing/pulling from an OCI registry
  #[clap(long, env = "OCI_PASSWORD", action)]
  pub(crate) password: Option<String>,

  /// Allows the use of HTTP registry connections to these registries.
  #[clap(long = "insecure-oci", action)]
  pub(crate) insecure_registries: Vec<String>,

  /// Allows the use of the 'latest' tag when fetching artifacts.
  #[clap(long = "allow-latest", action)]
  pub(crate) allow_latest: bool,
}

impl From<Options> for wick_config::FetchOptions {
  fn from(value: Options) -> Self {
    let mut fetch_options = wick_config::FetchOptions::default();
    fetch_options
      .set_allow_latest(value.allow_latest)
      .set_allow_insecure(value.insecure_registries.clone())
      .set_username(value.username)
      .set_password(value.password);

    fetch_options
  }
}

pub(crate) async fn pull(
  reference: String,
  oci_opts: wick_oci_utils::OciOptions,
) -> Result<wick_package::WickPackage, anyhow::Error> {
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
