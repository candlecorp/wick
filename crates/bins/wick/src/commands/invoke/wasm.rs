use std::collections::HashSet;
use std::time::SystemTime;

use anyhow::Result;
use wick_component_cli::parse_args;
use wick_component_wasm::collection::Collection;
use wick_component_wasm::helpers::WickWasmModule;
use wick_packet::{Entity, InherentData, Invocation, Observer, Packet, PacketStream};
use wick_rpc::RpcHandler;

use crate::utils;

pub(crate) async fn handle_command(opts: super::InvokeCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WickWasmModule::from_slice(&bytes)?;

  let collection = Collection::try_load(&component, 1, None, Some((opts.wasi).into()), None)?;

  let mut check_stdin = !opts.no_input && opts.data.is_empty() && opts.args.is_empty();
  if let Some(metadata) = component.token.claims.metadata {
    let target_component = metadata
      .interface
      .operations
      .iter()
      .find(|op| op.name == opts.component);

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
    todo!();
    // if atty::is(atty::Stream::Stdin) {
    //   eprintln!("No input passed, reading from <STDIN>. Pass --no-input to disable.");
    // }
    // let reader = io::BufReader::new(io::stdin());
    // let mut lines = reader.lines();
    // while let Some(line) = lines.next_line().await? {
    //   debug!("STDIN:'{}'", line);
    //   let mut payload = TransportMap::from_json_output(&line)?;
    //   payload.transpose_output_name();

    //   let invocation = Invocation::new(Entity::client("vow"), Entity::local(&opts.component), payload, None);

    //   let stream = collection.invoke(invocation).await.context("Component panicked")?;
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

    let invocation = Invocation::new(Entity::client("wick"), Entity::local(&opts.component), inherent_data);

    let stream = collection.invoke(invocation, stream).await?;
    utils::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  Ok(())
}
