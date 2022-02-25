use std::path::PathBuf;

use clap::Args;
use nkeys::KeyPairType;
use vino_par::make_archive;
use vino_rpc::ProviderSignature;
use vino_wascap::ClaimsOptions;

use crate::keys::{extract_keypair, GenerateCommon};
use crate::Result;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  /// Location of the binary to pack.
  pub(crate) binpath: PathBuf,

  /// Location of the interface JSON.
  pub(crate) interface_path: PathBuf,

  /// The destination file path.
  #[clap(short = 'o', long = "output", default_value = "./bundle.tar")]
  pub(crate) output: PathBuf,

  #[clap(short, long = "rev")]
  pub(crate) rev: Option<u32>,

  #[clap(short, long = "ver")]
  pub(crate) ver: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Pack artifact");

  let subject_kp = extract_keypair(
    Some(opts.binpath.to_string_lossy().to_string()),
    opts.common.directory.clone(),
    KeyPairType::Module,
  )?;

  let issuer_kp = extract_keypair(
    Some(opts.binpath.to_string_lossy().to_string()),
    opts.common.directory.clone(),
    KeyPairType::Account,
  )?;

  let binbytes = tokio::fs::read(&opts.binpath).await?;
  let signature_json = tokio::fs::read_to_string(&opts.interface_path).await?;
  let signature: ProviderSignature = serde_json::from_str(&signature_json)?;
  let options = ClaimsOptions {
    revision: opts.rev,
    version: opts.ver,
    expires_in_days: opts.common.expires_in_days,
    not_before_days: opts.common.not_before,
  };

  let bytes = make_archive(&*binbytes, &signature, options, &subject_kp, &issuer_kp)?;

  tokio::fs::write(&opts.output, &bytes).await?;

  Ok(())
}
