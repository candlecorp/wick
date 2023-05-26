use std::fs::File;
use std::io::Read;

use anyhow::Result;
use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct WasmInspectCommand {
  /// WebAssembly module location.
  #[clap(action)]
  pub(crate) module: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: WasmInspectCommand,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<()> {
  let _enter = span.enter();
  let mut file = File::open(opts.module)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  // Extract will return an error if it encounters an invalid hash in the claims
  let claims = wick_wascap::extract_claims(&buf)?;
  match claims {
    Some(claims) => println!("{}", serde_json::to_string(&claims)?),
    None => error!("Could not find any claims in the passed module"),
  }

  Ok(())
}
