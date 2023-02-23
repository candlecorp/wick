use std::collections::HashSet;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use wasmflow_collection_cli::parse_args;
use wasmflow_entity::Entity;
use wasmflow_packet_stream::{InherentData, Invocation, Observer, Packet, PacketStream};

use crate::utils;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RpcInvokeCommand {
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

pub(crate) async fn handle(opts: RpcInvokeCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut client = wasmflow_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let origin = Entity::client(crate::BIN_NAME);
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
    todo!();
    // if atty::is(atty::Stream::Stdin) {
    //   eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    // }
    // let reader = BufReader::new(io::stdin());
    // let mut lines = reader.lines();
    // while let Some(line) = lines.next_line().await? {
    //   let stream = client
    //     .invoke_from_json(origin.clone(), target.clone(), &line, !opts.raw, inherent_data)
    //     .await?;
    //   utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
    // }
  } else {
    let data = Packet::from_kv_json(&opts.data)?;

    let args = parse_args(&opts.args)?;
    let (tx, stream) = PacketStream::new_channels();
    let mut seen_ports = HashSet::new();
    for packet in args {
      seen_ports.insert(packet.port_name().to_owned());
      tx.send(packet)?;
    }
    for packet in data {
      seen_ports.insert(packet.port_name().to_owned());
      tx.send(packet)?;
    }
    for port in seen_ports {
      tx.send(Packet::done(port))?;
    }

    let invocation = Invocation::new(origin, target, inherent_data);
    trace!("issuing invocation");
    let stream = client.invoke(invocation, stream).await?;
    trace!("server responsed");
    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  Ok(())
}
