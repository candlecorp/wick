use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use tracing::Instrument;
use wick_config::WickConfiguration;
use wick_wascap::{sign_buffer_with_claims, ClaimsOptions};

use crate::keys::{get_module_keys, GenerateCommon};
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// WebAssembly module location.
  #[clap(action)]
  pub(crate) source: String,

  /// File path to the JSON representation of the module's interface.
  #[clap(action)]
  pub(crate) interface: String,

  /// Destination for signed module. If omitted, the signed module will have a .signed.wasm extension.
  #[clap(short = 'd', long = "destination", action)]
  destination: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  /// Version to embed in the module.
  #[clap(long, action)]
  ver: Option<String>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| {
    debug!("Signing module");
    debug!("Reading from {}", opts.interface);
  });

  let interface = WickConfiguration::fetch(&opts.interface, wick_config::FetchOptions::default())
    .instrument(span.clone())
    .await?
    .into_inner()
    .try_component_config()?;

  let mut source_file = File::open(&opts.source).unwrap();
  let mut buf = Vec::new();
  source_file.read_to_end(&mut buf).unwrap();

  let (account, subject) = get_module_keys(
    interface.name().map(|s| s.as_str()),
    opts.common.directory,
    opts.common.signer,
    opts.common.subject,
  )
  .await?;

  let sig = interface.signature()?;

  span.in_scope(|| Ok::<_, anyhow::Error>(debug!(signature = %serde_json::to_string(&sig)?, "component signature")))?;

  let signed = sign_buffer_with_claims(
    &buf,
    interface.signature()?,
    &subject,
    &account,
    &ClaimsOptions::v1(opts.ver, opts.common.expires_in_days, opts.common.wait),
  )?;

  let destination = match opts.destination.clone() {
    Some(d) => d,
    None => {
      let path = PathBuf::from(opts.source.clone())
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      let module_name = PathBuf::from(opts.source)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      // If path is empty, user supplied module in current directory
      if path.is_empty() {
        format!("./{}.signed.wasm", module_name)
      } else {
        format!("{}/{}.signed.wasm", path, module_name)
      }
    }
  };
  span.in_scope(|| debug!("Destination : {}", destination));

  let mut outfile = File::create(&destination).unwrap();

  span.in_scope(|| match outfile.write(&signed) {
    Ok(_) => {
      info!("Successfully signed {}", destination,);
      Ok(StructuredOutput::new(
        format!("Successfully signed: {}", destination),
        json!({
          "file": destination
        }),
      ))
    }
    Err(e) => {
      bail!("Error signing: {}", e)
    }
  })
}
