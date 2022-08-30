use std::str::FromStr;

use clap::Args;
use wasmflow_manifest::Permissions;

#[derive(Clone, Debug)]
struct StringPair(String, String);

impl FromStr for StringPair {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.split_once(':')
      .map(|(to, from)| StringPair(to.to_owned(), from.to_owned()))
      .ok_or_else(|| anyhow!("WASI directories need to be string pairs split by a colon, e.g. /to/dir:/from/dir"))
  }
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WasiOptions {
  /// Directories to expose to the WASM module via WASI. Ignored if loading a manifest.
  #[clap(long = "dirs", value_parser)]
  wasi_dir: Vec<StringPair>,
}

impl From<WasiOptions> for Permissions {
  fn from(opts: WasiOptions) -> Self {
    let dirs = opts.wasi_dir.into_iter().map(|v| (v.0, v.1)).collect();
    Self { dirs }
  }
}
