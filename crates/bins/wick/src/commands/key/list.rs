use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::keys::get_key_files;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct KeyListCommand {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// Location of key files. Defaults to $WICK_KEYS ($HOME/.wick/keys or %USERPROFILE%/.wick/keys).
  #[clap(long = "directory", env = "WICK_KEYS", action)]
  pub(crate) directory: Option<PathBuf>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: KeyListCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  debug!("Listing keys");
  let (dir, keys) = get_key_files(opts.directory)?;
  info!("Listing keys in {}", dir.to_string_lossy());

  for key in keys {
    println!("{}", key);
  }

  Ok(())
}
