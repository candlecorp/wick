use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::keys::GenerateCommon;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryPushCommand {
  #[clap(flatten)]
  pub(crate) logging: wick_logger::LoggingOptions,

  /// OCI reference to push to.
  #[clap(action)]
  pub(crate) reference: Option<String>,

  /// OCI artifact to push.
  #[clap(action)]
  pub(crate) source: PathBuf,

  #[clap(short, long = "rev", action)]
  pub(crate) rev: Option<u32>,

  #[clap(short, long = "ver", action)]
  pub(crate) ver: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: RegistryPushCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Push artifact");

  let mut package = wick_package::WickPackage::from_path(&opts.source).await?;
  let oci_opts = wick_oci_utils::OciOptions::default()
    .allow_insecure(opts.oci_opts.insecure_registries)
    .allow_latest(true)
    .username(opts.oci_opts.username)
    .password(opts.oci_opts.password);

  let reference = match opts.reference {
    Some(reference) => reference,
    None => package.registry_reference().unwrap().to_owned(),
  };

  info!("Pushing artifact...");
  debug!(options=?oci_opts, reference= &reference, "pushing reference");

  let _result = package.push(&reference, &oci_opts).await?;

  Ok(())
}
