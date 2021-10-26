use futures::StreamExt;
use structopt::StructOpt;
use tokio::io::{
  self,
  AsyncBufReadExt,
  BufReader,
};
use vino_entity::Entity;
use vino_transport::{
  TransportMap,
  TransportStream,
  TransportWrapper,
};

use crate::error::ControlError;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub(crate) connection: super::ConnectOptions,

  /// Don't read input from STDIN.
  #[structopt(long = "no-input")]
  pub(crate) no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Schematic to invoke.
  pub(crate) schematic: String,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,
}

pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut client = vino_rpc::make_rpc_client(
    opts.connection.address,
    opts.connection.port,
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let origin = Entity::client("vinoc");

  if opts.data.is_empty() && !opts.no_input {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    }
    let reader = BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines
      .next_line()
      .await
      .map_err(ControlError::ReadLineFailed)?
    {
      let stream = client
        .invoke_from_json(origin.url(), opts.schematic.clone(), &line, !opts.raw)
        .await?;
      print_stream_json(stream, opts.raw).await?;
    }
  } else {
    let mut payload = TransportMap::from_kv_json(&opts.data)?;
    if !opts.raw {
      payload.transpose_output_name();
    }
    let stream = client
      .invoke(origin.url(), opts.schematic.clone(), payload)
      .await?;
    print_stream_json(stream, opts.raw).await?;
  }

  Ok(())
}

async fn print_stream_json(mut stream: TransportStream, raw: bool) -> Result<()> {
  while let Some(message) = stream.next().await {
    let wrapper: TransportWrapper = message.into();
    if wrapper.payload.is_signal() && !raw {
      debug!(
        "Skipping signal '{}' in output, pass --raw to print.",
        wrapper.payload
      );
    } else {
      println!("{}", wrapper.into_json());
    }
  }
  Ok(())
}
