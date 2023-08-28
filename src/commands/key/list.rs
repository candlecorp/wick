use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;

use crate::keys::get_key_files;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Location of key files. Defaults to $WICK_KEYS ($HOME/.wick/keys or %USERPROFILE%/.wick/keys).
  #[clap(long = "directory", env = "WICK_KEYS", action)]
  pub(crate) directory: Option<PathBuf>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _enter = span.enter();
  debug!("Listing keys");
  let (dir, keys) = get_key_files(opts.directory)?;
  info!("Listing keys in {}", dir.to_string_lossy());

  let json = json!({"keys":keys});

  Ok(StructuredOutput::new(keys.join("\n"), json))
}
