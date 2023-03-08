use std::time::Duration;

use futures::StreamExt;
use logger::{LoggingGuard, LoggingOptions};
use wick_component_cli::options::DefaultCliOptions;
use wick_config::host_definition::{HttpConfig, MeshConfig};
use wick_config::ComponentConfiguration;
use wick_packet::PacketStream;

use crate::commands::FetchOptions;

#[allow(clippy::too_many_lines)]
pub(crate) fn merge_config(
  def: &ComponentConfiguration,
  local_cli_opts: &FetchOptions,
  server_cli_opts: Option<DefaultCliOptions>,
) -> ComponentConfiguration {
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

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn init_logger(opts: &LoggingOptions) -> crate::Result<LoggingGuard> {
  Ok(logger::init(&opts.name(crate::BIN_NAME)))
}

pub(crate) async fn print_stream_json(
  mut stream: PacketStream,
  filter: &[String],
  terse: bool,
  raw: bool,
) -> crate::Result<()> {
  if !filter.is_empty() {
    trace!("filtering only {:?}", filter);
  }
  while let Some(Ok(packet)) = stream.next().await {
    trace!(message = ?packet, "output");
    if (packet.is_done()) && !raw {
      continue;
    }
    if !filter.is_empty() && !filter.iter().any(|name| name == packet.port_name()) {
      tracing::debug!(port = %packet.port_name(), "filtering out");
      continue;
    }
    // TODO print actual json again
    if terse {

      // if let MessageTransport::Failure(err) = &packet.payload {
      //   return Err(anyhow::Error::msg(err.message().to_owned()));
      // }
      // let mut json = packet.payload.as_json();

      // json.as_object_mut().and_then(|o| o.remove("value")).map_or_else(
      //   || unreachable!("Message did not have an error nor a value: {}", json),
      //   |v| {
      //     println!(
      //       "{}",
      //       match v {
      //         serde_json::Value::String(s) => s,
      //         v => v.to_string(),
      //       }
      //     );
      //   },
      // );
    } else {
      println!("{}", serde_json::to_string(&packet)?);
    }
  }
  trace!("stream complete");
  Ok(())
}
