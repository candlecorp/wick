use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use seeded_random::Seed;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::parse_args;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_packet::{InherentData, Observer, Packet, PacketStream};

use crate::utils::{self, merge_config};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InvokeCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  location: String,

  /// Name of the operation to execute.
  #[clap(default_value = "default", action)]
  operation: String,

  /// Don't read input from STDIN.
  #[clap(long = "no-input", action)]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r', action)]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter", action)]
  filter: Vec<String>,

  /// A port=value string where value is JSON to pass as input.
  #[clap(long = "data", short = 'd', action)]
  data: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", short = 'o', action)]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle_command(mut opts: InvokeCommand) -> Result<()> {
  let mut logging = &mut opts.logging;
  if !(opts.info || logging.trace || logging.debug) {
    logging.quiet = true;
  }

  let _guard = wick_logger::init(&logging.name(crate::BIN_NAME));

  let fetch_options = if PathBuf::from(&opts.location).exists() {
    wick_config::config::FetchOptions::new()
  } else {
    wick_config::config::FetchOptions::new().artifact_dir(wick_xdg::Directories::GlobalCache.basedir()?)
  };
  let fetch_options = fetch_options
    .allow_latest(opts.oci.allow_latest)
    .allow_insecure(&opts.oci.insecure_registries);

  let manifest = WickConfiguration::fetch_all(&opts.location, fetch_options)
    .await?
    .try_component_config()?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let config = merge_config(&manifest, &opts.oci, Some(server_options));

  let component = opts.operation;

  let host_builder = ComponentHostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.start_engine(opts.seed.map(Seed::unsafe_new)).await?;

  let signature = host.get_signature()?;
  let target_schematic = signature.get_operation(&component);

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(target_schematic) = target_schematic {
    if target_schematic.inputs.is_empty() {
      check_stdin = false;
    }
  }

  let inherent_data = opts.seed.map(|seed| {
    InherentData::new(
      seed,
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    )
  });

  if check_stdin {
    todo!("STDIN support is not yet implemented.");
    // if atty::is(atty::Stream::Stdin) {
    //   eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    // }
    // let reader = io::BufReader::new(io::stdin());
    // let mut lines = reader.lines();

    // while let Some(line) = lines.next_line().await? {
    //   debug!("STDIN:'{}'", line);
    //   let mut payload = TransportMap::from_json_output(&line)?;
    //   if !opts.raw {
    //     payload.transpose_output_name();
    //   }

    //   let stream = host.request(&default_schematic, payload, inherent_data).await?;

    //   utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
    // }
  } else {
    let data = Packet::from_kv_json(&opts.data)?;

    let args = parse_args(&opts.args)?;
    trace!(args= ?args, "parsed CLI arguments");
    let (tx, stream) = PacketStream::new_channels();
    let mut seen_ports = HashSet::new();
    for packet in args {
      seen_ports.insert(packet.port().to_owned());
      tx.send(packet)?;
    }
    for packet in data {
      seen_ports.insert(packet.port().to_owned());
      tx.send(packet)?;
    }
    for port in seen_ports {
      tx.send(Packet::done(port))?;
    }

    let stream = host.request(&component, stream, inherent_data).await?;
    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }
  host.stop().await;

  Ok(())
}
