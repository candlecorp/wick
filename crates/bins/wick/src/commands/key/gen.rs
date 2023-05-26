use anyhow::Result;
use clap::Args;
use nkeys::{KeyPair, KeyPairType};

use crate::keys::GenerateCommon;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct KeyGenCommand {
  /// The type of key to create (e.g. account or module)
  #[clap(action)]
  keytype: KeyPairType,

  #[clap(flatten)]
  common: GenerateCommon,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: KeyGenCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  let _enter = span.enter();
  debug!("Generating {} key", crate::keys::keypair_type_to_string(&opts.keytype));

  let kp = KeyPair::new(opts.keytype);

  println!("Public key: {}", kp.public_key());
  println!("Seed: {}", kp.seed()?);

  Ok(())
}
