use structopt::StructOpt;
use tokio::io::{
  self,
  AsyncBufReadExt,
};
use tokio_stream::StreamExt;
use vino_provider::native::prelude::{
  BoxedTransportStream,
  Entity,
  TransportMap,
};
use vino_provider_cli::LoggingOptions;
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::message_transport::stream::map_to_json;

use crate::error::VowError;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[structopt(flatten)]
  pub(crate) logging: LoggingOptions,

  #[structopt(flatten)]
  pub(crate) pull: super::PullOptions,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  /// Name of the component to execute.
  component_name: String,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  vino_provider_cli::init_logging(&opts.logging)?;

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::try_from_module(&component, 1)?;

  if opts.data.is_empty() {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>");
    }
    let reader = io::BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
      debug!("STDIN:'{}'", line);
      let mut payload = TransportMap::from_json_str(&line)?;
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
    let mut payload = TransportMap::from_kv_json(&opts.data)?;
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

  Ok(())
}

async fn print_stream_json(stream: BoxedTransportStream, raw: bool) -> Result<()> {
  let mut json_stream = map_to_json(stream, raw);
  while let Some(message) = json_stream.next().await {
    println!("{}", message);
  }
  Ok(())
}
