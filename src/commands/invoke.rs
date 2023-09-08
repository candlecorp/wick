use std::collections::HashSet;
use std::time::SystemTime;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::parse_args;
use wick_host::Host;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketStream};

use crate::utils::{self, parse_config_string};
use crate::wick_host::build_host;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,

  #[clap(flatten)]
  pub(crate) operation: crate::options::component::OperationOptions,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Don't read input from STDIN.
  #[clap(long = "no-input", action)]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r', action)]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter", action)]
  filter: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", action)]
  short: bool,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let root_config = parse_config_string(opts.component.with.as_deref())?;
  let server_settings = DefaultCliOptions::default();

  let host = build_host(
    &opts.component.path,
    opts.oci,
    root_config,
    settings,
    opts.component.seed,
    Some(server_settings),
    span.clone(),
  )
  .await?;

  let mut path_parts = opts.operation.operation_name.split("::").collect::<Vec<_>>();

  if path_parts.is_empty() {
    return Err(anyhow::anyhow!(
      "Invalid operation name '{}', expected 'operation' or 'component::operation'",
      opts.operation.operation_name
    ));
  }

  let (path_parts, target) = if path_parts.len() == 1 {
    (None, Entity::local(path_parts[0]))
  } else {
    let op = path_parts.pop().unwrap();
    let component = path_parts.pop().unwrap();

    (Some(path_parts), Entity::operation(component, op))
  };

  let signature = host.get_signature(path_parts.as_deref(), Some(&target))?;

  let op_signature = signature.get_operation(target.operation_id()).ok_or_else(|| {
    anyhow::anyhow!(
      "Operation '{}' not found, reported operations are [{}]",
      target.operation_id(),
      signature
        .operations
        .iter()
        .map(|op| op.name())
        .collect::<Vec<_>>()
        .join(", ")
    )
  })?;

  let op_config = parse_config_string(opts.operation.op_with.as_deref())?;

  let check_stdin = if op_signature.inputs.is_empty() {
    false
  } else {
    !opts.no_input && opts.args.is_empty()
  };

  let inherent_data = opts.component.seed.map_or_else(InherentData::unsafe_default, |seed| {
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
    let args = parse_args(&opts.args, op_signature).map_err(|e| {
      anyhow!(
        "Failed to parse arguments for operation {}: {}",
        target.operation_id(),
        e
      )
    })?;
    span.in_scope(|| trace!(args= ?args, "parsed CLI arguments"));
    let mut packets = Vec::new();
    let mut seen_ports = HashSet::new();
    for packet in args {
      seen_ports.insert(packet.port().to_owned());
      packets.push(Ok(packet));
    }
    for port in seen_ports {
      packets.push(Ok(Packet::done(port)));
    }
    span.in_scope(|| debug!(cli_packets= ?packets, "wick invoke"));
    let stream = PacketStream::new(futures::stream::iter(packets));

    span.in_scope(|| info!(operation=%target,path= ?path_parts, "host loaded, invoking operation"));

    let invocation = Invocation::new(Entity::server(host.namespace()), target, stream, inherent_data, &span);

    let stream = host.invoke_deep(path_parts.as_deref(), invocation, op_config).await?;

    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  match host {
    wick_host::WickHost::App(_) => {}
    wick_host::WickHost::Component(host) => host.stop().await,
  }

  Ok(StructuredOutput::new("", json!({})))
}
