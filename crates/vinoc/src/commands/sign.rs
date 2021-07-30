use std::fs::File;
use std::io::{
  Read,
  Write,
};
use std::path::PathBuf;

use nkeys::KeyPairType;
use structopt::StructOpt;
use vino_types::signatures::ProviderSignature;
use vino_wascap::sign_buffer_with_claims;

use crate::keys::extract_keypair;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SignCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  /// File to read
  pub(crate) source: String,

  /// File path to the JSON representation of the actor interface
  pub(crate) interface: String,

  /// Destination for signed module. If this flag is not provided, the signed module will be placed in the same directory as the source with a "_s" suffix
  #[structopt(short = "d", long = "destination")]
  destination: Option<String>,

  #[structopt(flatten)]
  common: GenerateCommon,

  ///
  #[structopt(long)]
  ver: Option<String>,

  ///
  #[structopt(long)]
  rev: Option<u32>,
}

#[derive(Debug, Clone, StructOpt)]
struct GenerateCommon {
  /// Location of key files for signing. Defaults to $VINO_KEYS ($HOME/.vino/keys)
  #[structopt(long = "directory", env = "VINO_KEYS", hide_env_values = true)]
  directory: Option<String>,

  /// Indicates the token expires in the given amount of days. If this option is left off, the token will never expire
  #[structopt(short = "x", long = "expires")]
  expires_in_days: Option<u64>,

  /// Period in days that must elapse before this token is valid. If this option is left off, the token will be valid immediately
  #[structopt(short = "b", long = "nbf")]
  not_before_days: Option<u64>,
}

pub async fn handle_command(command: SignCommand) -> Result<()> {
  crate::utils::init_logger(&command.logging)?;
  debug!("Signing module");

  let json = std::fs::read_to_string(command.interface)?;

  let interface: ProviderSignature = serde_json::from_str(&json)?;

  let mut sfile = File::open(&command.source).unwrap();
  let mut buf = Vec::new();
  sfile.read_to_end(&mut buf).unwrap();

  let issuer = extract_keypair(
    Some(command.source.clone()),
    command.common.directory.clone(),
    KeyPairType::Account,
  )?;

  let subject = extract_keypair(
    Some(command.source.clone()),
    command.common.directory.clone(),
    KeyPairType::Module,
  )?;

  debug!("Signing module with interface : {:?}", interface);
  let signed = sign_buffer_with_claims(
    &buf,
    interface,
    &subject,
    &issuer,
    command.common.expires_in_days,
    command.common.not_before_days,
    command.ver,
    command.rev,
  )?;

  let destination = match command.destination.clone() {
    Some(d) => d,
    None => {
      let path = PathBuf::from(command.source.clone())
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      let module_name = PathBuf::from(command.source)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      // If path is empty, user supplied module in current directory
      if path.is_empty() {
        format!("./{}_s.wasm", module_name)
      } else {
        format!("{}/{}_s.wasm", path, module_name)
      }
    }
  };
  debug!("Destination : {}", destination);

  let mut outfile = File::create(&destination).unwrap();
  match outfile.write(&signed) {
    Ok(_) => {
      info!("Successfully signed {}", destination,);
    }
    Err(e) => {
      error!("Error signing: {}", e);
    }
  }

  Ok(())
}
