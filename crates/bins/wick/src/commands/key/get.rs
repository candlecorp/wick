use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// The filename to read (without path).
  #[clap(action)]
  path: PathBuf,

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
  let _span = span.enter();
  println!("Reading key: {}\n", opts.path.to_string_lossy());
  let kp = crate::keys::get_key(opts.directory, opts.path).await?;

  let lines = format!("Public key: {}\nPrivate seed: {}", kp.public_key(), kp.seed()?);

  let json = json!({"public_key":kp.public_key(),"private_seed":kp.seed()?});

  Ok(StructuredOutput::new(lines, json))
}
