use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use wasmflow_collection_cli::parse_args;
use wasmflow_sdk::v1::transport::TransportMap;
use wasmflow_sdk::v1::{Entity, InherentData, Invocation};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,

  /// Name of the component to execute.
  #[clap(action)]
  component: String,

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
  #[clap(long = "seed", short = 's', env = "WAFL_SEED", action)]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut client = wasmflow_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let origin = Entity::client("wafl");
  let target = Entity::local(&opts.component);

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

  let check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();

  if check_stdin {
    if atty::is(atty::Stream::Stdin) {
      eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    }
    let reader = BufReader::new(io::stdin());
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
      let stream = client
        .invoke_from_json(origin.clone(), target.clone(), &line, !opts.raw, inherent_data)
        .await?;
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
    let invocation = Invocation::new(origin, target, data_map, inherent_data);
    trace!("issuing invocation");
    let stream = client.invoke(invocation).await?;
    trace!("server responsed");
    cli_common::functions::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  Ok(())
}
