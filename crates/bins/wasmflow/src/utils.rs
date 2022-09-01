use std::time::Duration;

use wasmflow_collection_cli::options::DefaultCliOptions;
use wasmflow_manifest::host_definition::{HttpConfig, MeshConfig};
use wasmflow_manifest::WasmflowManifest;

use crate::commands::FetchOptions;

#[allow(clippy::too_many_lines)]
pub(crate) fn merge_config(
  def: &WasmflowManifest,
  local_cli_opts: &FetchOptions,
  server_cli_opts: Option<DefaultCliOptions>,
) -> WasmflowManifest {
  debug!("local_cli_opts.allow_latest {:?}", local_cli_opts.allow_latest);
  debug!("def.host.allow_latest {:?}", def.host().allow_latest);

  let mut merged_manifest = def.clone();
  let mut host_config = merged_manifest.host_mut();
  host_config.allow_latest = local_cli_opts.allow_latest || host_config.allow_latest;
  host_config.insecure_registries = vec![
    def.host().insecure_registries.clone(),
    local_cli_opts.insecure_registries.clone(),
  ]
  .concat();

  if let Some(cli_opts) = server_cli_opts {
    if let Some(to) = cli_opts.timeout {
      log_override("timeout", &mut host_config.timeout, Duration::from_millis(to));
    }
    if let Some(to) = cli_opts.id {
      log_override("id", &mut host_config.id, Some(to));
    }
    #[allow(clippy::option_if_let_else)]
    if let Some(manifest_opts) = host_config.rpc.as_mut() {
      if !manifest_opts.enabled {
        log_override("rpc.enabled", &mut manifest_opts.enabled, cli_opts.rpc_enabled);
      }
      if let Some(to) = cli_opts.rpc_address {
        log_override("rpc.address", &mut manifest_opts.address, Some(to));
      }
      if let Some(to) = cli_opts.rpc_port {
        log_override("rpc.port", &mut manifest_opts.port, Some(to));
      }
      if let Some(to) = cli_opts.rpc_pem {
        log_override("rpc.pem", &mut manifest_opts.pem, Some(to));
      }
      if let Some(to) = cli_opts.rpc_ca {
        log_override("rpc.ca", &mut manifest_opts.ca, Some(to));
      }
      if let Some(to) = cli_opts.rpc_key {
        log_override("rpc.key", &mut manifest_opts.key, Some(to));
      }
    } else {
      host_config.rpc = Some(HttpConfig {
        enabled: cli_opts.rpc_enabled,
        port: cli_opts.rpc_port,
        address: cli_opts.rpc_address,
        pem: cli_opts.rpc_pem,
        key: cli_opts.rpc_key,
        ca: cli_opts.rpc_ca,
      });
    };

    if let Some(mut manifest_opts) = host_config.mesh.as_mut() {
      if !manifest_opts.enabled {
        log_override("mesh.enabled", &mut manifest_opts.enabled, cli_opts.mesh.mesh_enabled);
      }
      if let Some(to) = cli_opts.mesh.nats_url {
        log_override("mesh.address", &mut manifest_opts.address, to);
      }
      if let Some(to) = cli_opts.mesh.nats_credsfile {
        log_override("mesh.creds_path", &mut manifest_opts.creds_path, Some(to));
      }
      if let Some(to) = cli_opts.mesh.nats_token {
        debug!("Overriding manifest value for 'host.mesh.token'");
        manifest_opts.token = Some(to);
      }
    } else if let Some(url) = cli_opts.mesh.nats_url {
      host_config.mesh = Some(MeshConfig {
        enabled: cli_opts.mesh.mesh_enabled,
        address: url,
        creds_path: cli_opts.mesh.nats_credsfile,
        token: cli_opts.mesh.nats_token,
      });
    }
  }
  merged_manifest
}

fn log_override<T: std::fmt::Debug>(field: &str, from: &mut T, to: T) {
  debug!(%field, ?from, ?to, "overriding manifest value");
  *from = to;
}
