use anyhow::Result;
use clap::Args;
use nkeys::{KeyPair, KeyPairType};

use crate::keys::GenerateCommon;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// The type of key to create (e.g. account or module)
  #[clap(action)]
  keytype: KeyPairType,

  #[clap(flatten)]
  common: GenerateCommon,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Generating {} key", crate::keys::keypair_type_to_string(&opts.keytype));

  let kp = KeyPair::new(opts.keytype);

  println!("Public key: {}", kp.public_key());
  println!("Seed: {}", kp.seed()?);

  Ok(())
}
