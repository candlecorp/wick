use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use tracing::Instrument;

use crate::oci::pull;
use crate::utils::reconcile_fetch_options;
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

  #[clap(flatten)]
  pub(crate) oci_opts: crate::options::oci::OciOptions,

  /// Write package files directly to output dir, don't use the wick heirachical structure.
  #[clap(long = "flattened", short = 'F', action)]
  pub(crate) flattened: bool,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let flattened = opts.flattened;
  let force = opts.oci_opts.force;
  let mut oci_opts = reconcile_fetch_options(&opts.reference, &settings, opts.oci_opts, Some(opts.output));
  oci_opts.set_ignore_manifest(true);
  if flattened {
    if !force {
      oci_opts.set_on_existing(wick_oci_utils::OnExisting::Error);
    }
    oci_opts.set_flatten(true);
  }

  span.in_scope(|| debug!(options=?oci_opts, reference= opts.reference, "pulling reference"));

  let package = pull(opts.reference, oci_opts).instrument(span.clone()).await?;

  let files = package
    .list_files()
    .iter()
    .map(|f| f.package_path().display().to_string())
    .collect::<Vec<_>>();

  let basedir = package.basedir().unwrap();

  Ok(StructuredOutput::new(
    format!(
      "Pulled file: \n{}",
      files
        .iter()
        .map(|n| format!("- {}", basedir.join(n).display()))
        .collect::<Vec<_>>()
        .join("\n")
    ),
    serde_json::json!({"files": files}),
  ))
}
