use clap::Args;

impl From<OciOptions> for wick_config::FetchOptions {
  fn from(value: OciOptions) -> Self {
    let mut fetch_options = wick_config::FetchOptions::default();
    fetch_options
      .set_allow_latest(value.allow_latest)
      .set_allow_insecure(value.insecure_registries.clone())
      .set_username(value.username)
      .set_password(value.password);

    fetch_options
  }
}

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct OciOptions {
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

  /// Force overwriting of files.
  #[clap(long = "force", short = 'f', action)]
  pub(crate) force: bool,
}
