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
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let oci_opts = reconcile_fetch_options(&opts.reference, &settings, opts.oci_opts, Some(opts.output));

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
