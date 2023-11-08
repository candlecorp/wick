use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use tracing::Instrument;

use crate::utils::get_auth_for_scope;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// OCI artifact to push.
  #[clap(action)]
  pub(crate) source: PathBuf,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::options::oci::OciOptions,

  /// Registry to use (overriding configured registry)
  #[clap(long = "registry", action)]
  pub(crate) registry: Option<String>,

  #[clap(long = "tag")]
  pub(crate) tags: Vec<String>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| debug!("push artifact"));

  let mut package = wick_package::WickPackage::from_path(None, &opts.source)
    .instrument(span.clone())
    .await?;

  let Some(registry) = package.registry_mut() else {
    span.in_scope(|| error!("no registry provided in package"));
    return Err(anyhow!("No registry provided in package"));
  };

  if let Some(reg_override) = opts.registry {
    registry.set_host(reg_override);
  }

  let configured_creds = settings.credentials.iter().find(|c| c.scope == registry.host());

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci_opts.username.as_deref(),
    opts.oci_opts.password.as_deref(),
  );

  let mut oci_opts = wick_oci_utils::OciOptions::default();
  oci_opts
    .set_allow_insecure(opts.oci_opts.insecure_registries)
    .set_allow_latest(true)
    .set_username(username)
    .set_password(password);

  let reference = package.registry_reference().unwrap(); // unwrap OK because we know we have a reg from above.

  span.in_scope(|| {
    info!(reference, "pushing artifact");
    debug!(options=?oci_opts, reference= &reference, "pushing reference");
  });

  let mut lines = Vec::new();

  let response = package.push(&reference, opts.tags, &oci_opts).await?;

  span.in_scope(|| info!(url=%response.reference, "manifest pushed"));
  for tag in &response.tags {
    span.in_scope(|| info!(url=%tag, "tag pushed"));
  }

  let json = json!({"manifest_url":&response.manifest_url, reference: &response.reference,"tags": &response.tags});

  lines.push(format!("Manifest URL: {}", response.manifest_url));
  lines.push(format!("Pushed reference: {}", response.reference));
  if !response.tags.is_empty() {
    lines.push(format!("Pushed tags:\n{}", response.tags.join("\n")));
  }

  Ok(StructuredOutput::new(lines.join("\n"), json))
}
