use std::path::PathBuf;

use structopt::StructOpt;
use tokio::io::{
  self,
  AsyncBufReadExt,
};
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::cli::{
  DefaultCliOptions,
  LatticeCliOptions,
};
use vino_runtime::prelude::StreamExt;
use vino_transport::message_transport::stream::map_to_json;
use vino_transport::{
  TransportMap,
  TransportStream,
};

use crate::utils::merge_config;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Turn on info logging.
  #[structopt(long = "info")]
  pub(crate) info: bool,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Manifest file.
  manifest: PathBuf,

  /// Default schematic to run.
  #[structopt()]
  default_schematic: Option<String>,
}

pub(crate) async fn handle_command(command: RunCommand) -> Result<String> {
  let mut logging = command.logging;
  if !(command.info || command.logging.trace || command.logging.debug) {
    logging.quiet = true;
  }
  logger::init(&logging);

  let config = HostDefinition::load_from_file(&command.manifest)?;

  let server_options = DefaultCliOptions {
    lattice: command.lattice,
    ..Default::default()
  };

  let mut config = merge_config(config, command.host, Some(server_options));
  if command.default_schematic.is_some() {
    config.default_schematic = command.default_schematic.unwrap();
  }
  let default_schematic = config.default_schematic.clone();

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network().await?;

  if command.data.is_empty() {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>");
    }
    let reader = io::BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
      debug!("STDIN:'{}'", line);
      let mut payload = TransportMap::from_json_str(&line)?;
      payload.transpose_output_name();
      let stream = host.request(&default_schematic, payload).await?;

      print_stream_json(stream, command.raw).await?;
    }
  } else {
    let mut payload = TransportMap::from_kv_json(&command.data)?;
    payload.transpose_output_name();
    let stream = host.request(&default_schematic, payload).await?;
    print_stream_json(stream, command.raw).await?;
  }

  Ok("Done".to_owned())
}

async fn print_stream_json(stream: TransportStream, raw: bool) -> Result<()> {
  let mut json_stream = map_to_json(stream, raw);
  while let Some(message) = json_stream.next().await {
    println!("{}", message);
  }
  Ok(())
}
