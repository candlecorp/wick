use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::options::get_auth_for_scope;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RegistryPushCommand {
  /// OCI artifact to push.
  #[clap(action)]
  pub(crate) source: PathBuf,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,

  #[clap(long = "latest", action)]
  pub(crate) latest: bool,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: RegistryPushCommand, settings: wick_settings::Settings) -> Result<()> {
  debug!("Push artifact");

  let mut package = wick_package::WickPackage::from_path(&opts.source).await?;

  let Some(registry) =  package.registry() else  {
      error!("No registry provided in package");
      return Err(anyhow!("No registry provided in package"));
  };

  let configured_creds = settings.credentials.iter().find(|c| c.scope == registry.host());

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci_opts.username.as_deref(),
    opts.oci_opts.password.as_deref(),
  );

  let oci_opts = wick_oci_utils::OciOptions::default()
    .allow_insecure(opts.oci_opts.insecure_registries)
    .allow_latest(true)
    .username(username)
    .password(password);

  let reference = package.registry_reference().unwrap(); // unwrap OK because we know we have a reg from above.

  info!(reference, "pushing artifact");
  debug!(options=?oci_opts, reference= &reference, "pushing reference");

  let url = package.push(&reference, &oci_opts).await?;
  info!(%url, "artifact pushed");
  if opts.latest {
    let reference = package.tagged_reference("latest").unwrap();

    info!(reference, "pushing artifact");
    debug!(options=?oci_opts, reference= &reference, "pushing reference");

    let url = package.push(&reference, &oci_opts).await?;
    info!(%url, "artifact pushed");
  }

  Ok(())
}
