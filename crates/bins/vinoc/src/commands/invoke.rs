use clap::Args;
use futures::StreamExt;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use vino_entity::Entity;
use vino_provider_cli::parse_args;
use vino_transport::{Invocation, TransportMap, TransportStream};

use crate::error::ControlError;
use crate::Result;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,

  /// Don't read input from STDIN.
  #[clap(long = "no-input")]
  pub(crate) no_input: bool,

  /// A port=value string where value is JSON to pass as input.
  #[clap(long, short)]
  data: Vec<String>,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long, short)]
  raw: bool,

  /// Component to invoke.
  pub(crate) component: String,

  /// Arguments to pass as inputs to a schematic.
  #[clap(last(true))]
  args: Vec<String>,
}

pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut client = vino_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let origin = Entity::client("vinoc");

  let check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();

  if check_stdin {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    }
    let reader = BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.map_err(ControlError::ReadLineFailed)? {
      let stream = client
        .invoke_from_json(origin.url(), opts.component.clone(), &line, !opts.raw)
        .await?;
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
    let invocation = Invocation::new(origin, Entity::component_direct(opts.component), data_map);
    let stream = client.invoke(invocation).await?;
    print_stream_json(stream, opts.raw).await?;
  }

  Ok(())
}

async fn print_stream_json(mut stream: TransportStream, raw: bool) -> Result<()> {
  while let Some(message) = stream.next().await {
    if message.payload.is_signal() && !raw {
      debug!("Skipping signal '{}' in output, pass --raw to print.", message.payload);
    } else {
      println!("{}", message.into_json());
    }
  }
  Ok(())
}
