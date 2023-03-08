use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use wick_grpctar::make_archive;
use wick_rpc::CollectionSignature;
use wick_wascap::ClaimsOptions;

use crate::io::{read_bytes, read_to_string, write_bytes};
use crate::keys::{get_module_keys, GenerateCommon};
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct BundlePackCommand {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// Location of the binary to pack.
  #[clap(action)]
  pub(crate) binpath: PathBuf,

  /// Location of the interface JSON.
  #[clap(action)]
  pub(crate) interface_path: PathBuf,

  /// The destination file path.
  #[clap(short = 'o', long = "output", default_value = "./bundle.tar", action)]
  pub(crate) output: PathBuf,

  #[clap(short, long = "rev", action)]
  pub(crate) rev: Option<u32>,

  #[clap(short, long = "ver", action)]
  pub(crate) ver: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: BundlePackCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Pack artifact");

  let (account, subject) = get_module_keys(
    Some(opts.binpath.to_string_lossy().to_string()),
    opts.common.directory,
    opts.common.signer,
    opts.common.subject,
  )
  .await?;

  let binbytes = read_bytes(&opts.binpath).await?;
  let signature_json = read_to_string(&opts.interface_path).await?;
  let signature: CollectionSignature = serde_json::from_str(&signature_json)?;
  let options = ClaimsOptions {
    revision: opts.rev,
    version: opts.ver,
    expires_in_days: opts.common.expires_in_days,
    not_before_days: opts.common.wait,
  };

  let bytes = make_archive(&*binbytes, &signature, options, &subject, &account)?;

  write_bytes(&opts.output, &bytes).await?;

  Ok(())
}
