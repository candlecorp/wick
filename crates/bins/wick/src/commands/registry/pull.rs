use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use tracing::Instrument;
use wick_oci_utils::OciOptions;

use crate::options::get_auth_for_scope;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// OCI reference to pull.
  #[clap(action)]
  pub(crate) reference: String,

  /// Directory to store the pulled artifacts.
  #[clap(action, default_value = ".")]
  pub(crate) output: PathBuf,

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
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let configured_creds = settings
    .credentials
    .iter()
    .find(|c| opts.reference.starts_with(&c.scope));

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
    .set_password(password)
    .set_overwrite(opts.force)
    .set_cache_dir(opts.output);

  span.in_scope(|| debug!(options=?oci_opts, reference= opts.reference, "pulling reference"));

  let pull_result = pull(opts.reference, oci_opts).instrument(span.clone()).await?;

  let files = pull_result
    .list_files()
    .iter()
    .map(|f| f.path().display().to_string())
    .collect::<Vec<_>>();

  Ok(StructuredOutput::new(
    format!("Pulled file: \n {}", files.join("\n")),
    serde_json::json!({"files": files}),
  ))
}
