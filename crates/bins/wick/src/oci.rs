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
  pub(crate) insecure_oci_registries: Vec<String>,
}
