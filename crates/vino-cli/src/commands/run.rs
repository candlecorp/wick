use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;

use crate::utils::merge_runconfig;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub nats: super::NatsOptions,

  #[structopt(flatten)]
  pub host: super::HostOptions,

  /// Turn on info logging
  #[structopt(long = "info")]
  pub info: bool,

  /// Default schematic to run
  #[structopt(long, short, env = "VINO_DEFAULT_SCHEMATIC")]
  pub default_schematic: Option<String>,

  /// Manifest file
  manifest: PathBuf,

  /// JSON data
  data: Option<String>,
}

pub async fn handle_command(command: RunCommand) -> Result<String> {
  let mut logging = command.logging;
  if !(command.info || command.logging.trace || command.logging.debug) {
    logging.quiet = true;
  }
  crate::utils::init_logger(&logging)?;

  let data = match command.data {
    None => {
      eprintln!("No input passed, reading from <STDIN>");
      let mut data = String::new();
      std::io::stdin().read_to_string(&mut data)?;
      data
    }
    Some(i) => i,
  };

  debug!("Received {} bytes of json", data.len());

  let json: HashMap<String, serde_json::value::Value> = serde_json::from_str(&data)?;

  let config = vino_host::HostDefinition::load_from_file(&command.manifest)?;

  let mut config = merge_runconfig(config, command.nats, command.host);
  if command.default_schematic.is_some() {
    config.default_schematic = command.default_schematic.unwrap();
  }

  let result = crate::run(config, json).await?;

  println!("{}", result);

  Ok("Done".to_owned())
}
