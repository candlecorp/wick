use structopt::StructOpt;
use tokio::io::{
  self,
  AsyncBufReadExt,
};
use tokio_stream::StreamExt;
use vino_provider::native::prelude::{
  BoxedTransportStream,
  Entity,
  MapWrapper,
  TransportMap,
};
use vino_provider_cli::utils::parse_args;
use vino_provider_cli::LoggingOptions;
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::message_transport::stream::map_to_json;

use super::WasiOptions;
use crate::error::VowError;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[structopt(flatten)]
  logging: LoggingOptions,

  #[structopt(flatten)]
  pull: super::PullOptions,

  /// Don't read input from STDIN.
  #[structopt(long = "no-input")]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,

  #[structopt(flatten)]
  wasi: WasiOptions,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  /// Name of the component to execute.
  component_name: String,

  /// Arguments to pass as inputs to a schematic.
  #[structopt(set = structopt::clap::ArgSettings::Last)]
  args: Vec<String>,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = vino_provider_cli::init_logging(&opts.logging.name("vow"));

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::try_load(&component, 1, None, Some((&opts.wasi).into()), None)?;

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(metadata) = component.token.claims.metadata {
    let target_component = metadata.interface.components.get(&opts.component_name);

    if let Some(target_component) = target_component {
      if target_component.inputs.is_empty() {
        check_stdin = false;
      }
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
      let mut payload = TransportMap::from_json_output(&line)?;
      payload.transpose_output_name();
      let stream = provider
        .invoke(
          Entity::component_direct(opts.component_name.clone()),
          payload,
        )
        .await
        .map_err(VowError::ComponentPanic)?;
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

    let stream = provider
      .invoke(
        Entity::component_direct(opts.component_name.clone()),
        data_map,
      )
      .await
      .map_err(VowError::ComponentPanic)?;
    print_stream_json(stream, opts.raw).await?;
  }

  Ok(())
}

async fn print_stream_json(stream: BoxedTransportStream, raw: bool) -> Result<()> {
  let mut json_stream = map_to_json(stream, raw);
  while let Some(message) = json_stream.next().await {
    println!("{}", message);
  }
  Ok(())
}
