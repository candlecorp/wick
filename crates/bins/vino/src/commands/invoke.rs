use std::time::SystemTime;

use clap::Args;
use tokio::io::{self, AsyncBufReadExt};
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::options::{DefaultCliOptions, LatticeCliOptions};
use vino_provider_cli::parse_args;
use vino_random::Seed;
use vino_transport::{InherentData, TransportMap};
use vino_types::MapWrapper;

use crate::utils::merge_config;
use crate::Result;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InvokeCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[clap(flatten)]
  pub(crate) host: super::HostOptions,

  /// Turn on info logging.
  #[clap(long = "info")]
  pub(crate) info: bool,

  /// Manifest file or OCI url.
  manifest: String,

  // *****************************************************************
  // Everything below is copied from common-cli-options::RunOptions
  // Flatten doesn't work with positional args...
  //
  // TODO: Eliminate the need for copy/pasting
  // *****************************************************************
  /// Name of the component to execute.
  #[clap(default_value = "default")]
  component: String,

  /// Don't read input from STDIN.
  #[clap(long = "no-input")]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r')]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter")]
  filter: Vec<String>,

  /// A port=value string where value is JSON to pass as input.
  #[clap(long = "data", short = 'd')]
  data: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", short = 'o')]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "VINO_SEED")]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a schematic.
  #[clap(last(true))]
  args: Vec<String>,
}

pub(crate) async fn handle_command(opts: InvokeCommand) -> Result<()> {
  let mut logging = opts.logging;
  if !(opts.info || logging.trace || logging.debug) {
    logging.quiet = true;
  }
  let _guard = logger::init(&logging.name("vino"));

  debug!(args = ?opts.args, "rest args");

  let manifest_src = vino_loader::get_bytes(&opts.manifest, opts.host.allow_latest, &opts.host.insecure_registries)
    .await
    .map_err(|e| crate::error::VinoError::ManifestLoadFail(e.to_string()))?;

  let manifest = HostDefinition::load_from_bytes(Some(opts.manifest), &manifest_src)?;

  let server_options = DefaultCliOptions {
    lattice: opts.lattice,
    ..Default::default()
  };

  let mut config = merge_config(manifest, &opts.host, Some(server_options));
  if config.default_schematic.is_none() {
    config.default_schematic = Some(opts.component);
  }

  let default_schematic = config.default_schematic.clone().unwrap();

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network(opts.seed.map(Seed::unsafe_new)).await?;

  let signature = host.get_signature()?;
  let target_schematic = signature.get_component(&default_schematic);

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
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    }
    let reader = io::BufReader::new(io::stdin());
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
      debug!("STDIN:'{}'", line);
      let mut payload = TransportMap::from_json_output(&line)?;
      if !opts.raw {
        payload.transpose_output_name();
      }

      let stream = host.request(&default_schematic, payload, inherent_data).await?;

      cli_common::functions::print_stream_json(Box::pin(stream), &opts.filter, opts.short, opts.raw).await?;
    }
  } else {
    let mut data_map = TransportMap::from_kv_json(&opts.data)?;

    let mut rest_arg_map = parse_args(&opts.args)?;
    if !opts.raw {
      data_map.transpose_output_name();
      rest_arg_map.transpose_output_name();
    }
    data_map.merge(rest_arg_map);

    let stream = host.request(&default_schematic, data_map, inherent_data).await?;
    cli_common::functions::print_stream_json(Box::pin(stream), &opts.filter, opts.short, opts.raw).await?;
  }
  host.stop().await;

  Ok(())
}
