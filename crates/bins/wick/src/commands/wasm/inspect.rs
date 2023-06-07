use std::fs::File;
use std::io::Read;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// WebAssembly module location.
  #[clap(action)]
  pub(crate) module: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _enter = span.enter();
  let mut file = File::open(opts.module)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  // Extract will return an error if it encounters an invalid hash in the claims
  let claims = wick_wascap::extract_claims(&buf)?;
  match claims {
    Some(claims) => Ok(StructuredOutput::new(
      serde_json::to_string(&claims)?,
      json!({"claims":claims}),
    )),
    None => {
      bail!("Could not find any claims in the passed module")
    }
  }
}
