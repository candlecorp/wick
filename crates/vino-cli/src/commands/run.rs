use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;
use vino_runtime::prelude::StreamExt;

use crate::utils::merge_runconfig;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Turn on info logging.
  #[structopt(long = "info")]
  pub(crate) info: bool,

  /// Default schematic to run.
  #[structopt(long, short, env = "VINO_DEFAULT_SCHEMATIC")]
  pub(crate) default_schematic: Option<String>,

  /// Manifest file.
  manifest: PathBuf,

  /// JSON data.
  data: Option<String>,
}

pub(crate) async fn handle_command(command: RunCommand) -> Result<String> {
  let mut logging = command.logging;
  if !(command.info || command.logging.trace || command.logging.debug) {
    logging.quiet = true;
  }
  logger::init(&logging);

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

  let mut config = merge_runconfig(config, command.host);
  if command.default_schematic.is_some() {
    config.default_schematic = command.default_schematic.unwrap();
  }

  let mut result = vino_host::run::run(config, json).await?;
  while let Some(message) = result.next().await {
    if message.payload.is_signal() {
      debug!(
        "Skipping signal '{}' on port '{}'",
        message.payload, message.port
      );
    } else {
      println!("{}", message.payload.into_json());
    }
  }

  Ok("Done".to_owned())
}
