use anyhow::Result;
use wasmflow_host::HostBuilder;
use wasmflow_manifest::WasmflowManifest;

use crate::utils::merge_config;

pub(crate) async fn handle_command(opts: super::ServeCommand, bytes: Vec<u8>) -> Result<()> {
  let manifest = WasmflowManifest::load_from_bytes(Some(opts.location), &bytes)?;

  let config = merge_config(&manifest, &opts.fetch, Some(opts.cli));

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();

  host.start(Some(0)).await?;
  info!("Host started");
  #[allow(clippy::option_if_let_else)]
  match host.get_server_info() {
    Some(info) => {
      wasmflow_collection_cli::print_info(info);
    }
    None => {
      warn!("No server information available, did you intend to start a host without GRPC or a mesh connection?");
    }
  };
  info!("Waiting for Ctrl-C");
  let _ = tokio::signal::ctrl_c().await;
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok(())
}
