use structopt::StructOpt;
use tokio::io::{
  self,
  AsyncBufReadExt,
  BufReader,
};
use vino_rpc::rpc::Output;
use vino_transport::TransportWrapper;

use crate::rpc_client::rpc_client;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct InvokeCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Schematic to invoke.
  pub schematic: String,

  /// JSON map of data to use as the component payload.
  data: Vec<String>,
}

pub async fn handle_command(opts: InvokeCommand) -> Result<()> {
  crate::utils::init_logger(&opts.logging)?;
  let mut client = rpc_client(
    opts.connection.address,
    opts.connection.port,
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  if opts.data.is_empty() {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>");
    }
    let reader = BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
      let stream = client
        .invoke_from_json(opts.schematic.clone(), &line, opts.raw)
        .await?;
      print_stream_json(stream, opts.raw).await?;
    }
  } else {
    for message in opts.data {
      let stream = client
        .invoke_from_json(opts.schematic.clone(), &message, opts.raw)
        .await?;
      print_stream_json(stream, opts.raw).await?;
    }
  }

  Ok(())
}

async fn print_stream_json(mut stream: tonic::Streaming<Output>, raw: bool) -> Result<()> {
  while let Some(message) = stream.message().await? {
    let wrapper: TransportWrapper = message.into();
    if wrapper.payload.is_signal() && !raw {
      debug!("STDIN is interactive, skipping signal: {}", wrapper.payload);
    } else {
      println!("{}", wrapper.into_json());
    }
  }
  Ok(())
}
