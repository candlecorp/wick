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
    let fetch_options = wick_config::FetchOptions::default();
    let mut fetch_options = fetch_options
      .allow_latest(value.allow_latest)
      .allow_insecure(&value.insecure_registries);
    if let Some(username) = value.username {
      fetch_options = fetch_options.oci_username(username);
    }
    if let Some(password) = value.password {
      fetch_options = fetch_options.oci_password(password);
    }

    fetch_options
  }
}
