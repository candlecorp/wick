use std::time::SystemTime;

use clap::Args;
use tokio::io::{self, AsyncBufReadExt};
use wasmflow_entity::Entity;
use vino_provider_cli::{parse_args, LoggingOptions};
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::TransportMap;
use wasmflow_invocation::{InherentData, Invocation};

use super::WasiOptions;
use crate::error::VowError;
use crate::Result;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  /// Path or URL to WebAssembly binary.
  wasm: String,

  #[clap(flatten)]
  logging: LoggingOptions,

  #[clap(flatten)]
  pull: super::PullOptions,

  #[clap(flatten)]
  wasi: WasiOptions,

  /// Name of the component to execute.
  component: String,

  // *****************************************************************
  // Everything below is copied from common-cli-options::RunOptions
  // Flatten doesn't work with positional args...
  //
  // TODO: Eliminate the need for copy/pasting
  // *****************************************************************
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

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = vino_provider_cli::init_logging(&opts.logging.name("vow"));

  debug!("Loading wasm {}", opts.wasm);
  let component = vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure).await?;

  let provider = Provider::try_load(&component, 1, None, Some((&opts.wasi).into()), None)?;

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(metadata) = component.token.claims.metadata {
    let target_component = metadata.interface.components.get(&opts.component);

    if let Some(target_component) = target_component {
      if target_component.inputs.is_empty() {
        check_stdin = false;
      }
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
      payload.transpose_output_name();

      let invocation = Invocation::new(Entity::client("vow"), Entity::local(&opts.component), payload, None);

      let stream = provider.invoke(invocation).await.map_err(VowError::ComponentPanic)?;
      cli_common::functions::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
    }
  } else {
    let mut data_map = TransportMap::from_kv_json(&opts.data)?;

    let mut rest_arg_map = parse_args(&opts.args)?;
    if !opts.raw {
      data_map.transpose_output_name();
      rest_arg_map.transpose_output_name();
    }
    data_map.merge(rest_arg_map);

    let invocation = Invocation::new(
      Entity::client("vow"),
      Entity::local(&opts.component),
      data_map,
      inherent_data,
    );

    let stream = provider.invoke(invocation).await.map_err(VowError::ComponentPanic)?;
    cli_common::functions::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  Ok(())
}
