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
use vino_provider_cli::utils::parse_args;
use vino_runtime::prelude::StreamExt;
use vino_transport::message_transport::stream::map_to_json;
use vino_transport::{
  TransportMap,
  TransportStream,
};
use vino_types::signatures::MapWrapper;

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

  /// Don't read input from STDIN.
  #[structopt(long = "no-input")]
  pub(crate) no_input: bool,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Manifest file or OCI url.
  manifest: String,

  /// Default schematic to run.
  default_schematic: Option<String>,

  /// Arguments to pass as inputs to a schematic.
  #[structopt(set = structopt::clap::ArgSettings::Last)]
  args: Vec<String>,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let mut logging = opts.logging;
  if !(opts.info || opts.logging.trace || opts.logging.debug) {
    logging.quiet = true;
  }
  logger::init(&logging);
  debug!("rest: {:?}", opts.args);

  let manifest_src = vino_loader::get_bytes(
    &opts.manifest,
    opts.host.allow_latest,
    &opts.host.insecure_registries,
  )
  .await
  .map_err(|e| crate::error::VinoError::ManifestLoadFail(e.to_string()))?;

  let manifest = HostDefinition::load_from_bytes(&manifest_src)?;

  let server_options = DefaultCliOptions {
    lattice: opts.lattice,
    ..Default::default()
  };

  let mut config = merge_config(manifest, opts.host, Some(server_options));
  if opts.default_schematic.is_some() {
    config.default_schematic = opts.default_schematic.unwrap();
  }
  let default_schematic = config.default_schematic.clone();

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network().await?;

  let signature = host.get_signature().await?;
  let target_schematic = signature.get_component(&default_schematic);

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(target_schematic) = target_schematic {
    if target_schematic.inputs.is_empty() {
      check_stdin = false;
    }
  }

  if check_stdin {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    }
    let reader = io::BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
      debug!("STDIN:'{}'", line);
      let mut payload = TransportMap::from_json_str(&line)?;
      if !opts.raw {
        payload.transpose_output_name();
      }
      let stream = host.request(&default_schematic, payload, None).await?;

      print_stream_json(stream, opts.raw).await?;
    }
  } else {
    let mut data_map = TransportMap::from_kv_json(&opts.data)?;

    let mut rest_arg_map = parse_args(&opts.args)?;
    if !opts.raw {
      data_map.transpose_output_name();
      rest_arg_map.transpose_output_name();
    }
    data_map.merge(rest_arg_map);

    let stream = host.request(&default_schematic, data_map, None).await?;
    print_stream_json(stream, opts.raw).await?;
  }

  Ok(())
}

async fn print_stream_json(stream: TransportStream, raw: bool) -> Result<()> {
  let mut json_stream = map_to_json(stream, raw);
  while let Some(message) = json_stream.next().await {
    println!("{}", message);
  }
  Ok(())
}
