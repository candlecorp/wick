use anyhow::Result;
use clap::Args;
use nkeys::{KeyPair, KeyPairType};
use serde_json::json;
use structured_output::StructuredOutput;

use crate::keys::GenerateCommon;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// The type of key to create (e.g. account or module)
  #[clap(action)]
  keytype: KeyPairType,

  #[clap(flatten)]
  common: GenerateCommon,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _enter = span.enter();
  debug!("generating {} key", crate::keys::keypair_type_to_string(&opts.keytype));

  let kp = KeyPair::new(opts.keytype);

  let lines = format!("Public key: {}\nPrivate seed: {}", kp.public_key(), kp.seed()?);

  let json = json!({"public_key":kp.public_key(),"private_seed":kp.seed()?});
  Ok(StructuredOutput::new(lines, json))
}
