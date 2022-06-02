use std::time::SystemTime;

use anyhow::{Context, Result};
use tokio::io::{self, AsyncBufReadExt};
use wasmflow_collection_cli::parse_args;
use wasmflow_collection_wasm::helpers::WapcModule;
use wasmflow_collection_wasm::provider::Provider;
use wasmflow_entity::Entity;
use wasmflow_invocation::{InherentData, Invocation};
use wasmflow_rpc::RpcHandler;
use wasmflow_transport::TransportMap;

pub(crate) async fn handle_command(opts: super::InvokeCommand, bytes: Vec<u8>) -> Result<()> {
  let component = WapcModule::from_slice(&bytes)?;

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

      let stream = provider.invoke(invocation).await.context("Component panicked")?;
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

    let stream = provider.invoke(invocation).await.context("Component panicked")?;
    cli_common::functions::print_stream_json(stream, &opts.filter, opts.short, opts.raw).await?;
  }

  Ok(())
}
