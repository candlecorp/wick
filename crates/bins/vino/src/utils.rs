use std::time::Duration;

use vino_manifest::host_definition::{HostConfig, HostDefinition, HttpConfig, LatticeConfig};
use vino_provider_cli::options::DefaultCliOptions;

use crate::commands::HostOptions;

#[allow(clippy::too_many_lines)]
#[must_use]
pub(crate) fn merge_config(
  def: HostDefinition,
  local_cli_opts: &HostOptions,
  server_cli_opts: Option<DefaultCliOptions>,
) -> HostDefinition {
  debug!("local_cli_opts.allow_latest {:?}", local_cli_opts.allow_latest);
  debug!("def.host.allow_latest {:?}", def.host.allow_latest);
  let mut host_config = HostConfig {
    allow_latest: local_cli_opts.allow_latest || def.host.allow_latest,
    insecure_registries: vec![def.host.insecure_registries, local_cli_opts.insecure_registries.clone()].concat(),
    timeout: def.host.timeout,
    id: def.host.id,
    ..Default::default()
  };

  if let Some(cli_opts) = server_cli_opts {
    if let Some(to) = cli_opts.timeout {
      log_override("timeout", &mut host_config.timeout, Duration::from_millis(to));
    }
    if let Some(to) = cli_opts.id {
      log_override("id", &mut host_config.id, Some(to));
    }
    #[allow(clippy::option_if_let_else)]
    let rpc_opts = if let Some(mut manifest_opts) = def.host.rpc {
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
      manifest_opts
    } else {
      HttpConfig {
        enabled: cli_opts.rpc_enabled,
        port: cli_opts.rpc_port,
        address: cli_opts.rpc_address,
        pem: cli_opts.rpc_pem,
        key: cli_opts.rpc_key,
        ca: cli_opts.rpc_ca,
      }
    };
    host_config.rpc = Some(rpc_opts);

    #[allow(clippy::option_if_let_else)]
    let http_opts = if let Some(mut manifest_opts) = def.host.http {
      if !manifest_opts.enabled {
        log_override("http.enabled", &mut manifest_opts.enabled, cli_opts.http_enabled);
      }
      if let Some(to) = cli_opts.http_address {
        log_override("http.address", &mut manifest_opts.address, Some(to));
      }
      if let Some(to) = cli_opts.http_port {
        log_override("http.port", &mut manifest_opts.port, Some(to));
      }
      if let Some(to) = cli_opts.http_pem {
        log_override("http.pem", &mut manifest_opts.pem, Some(to));
      }
      if let Some(to) = cli_opts.http_ca {
        log_override("http.ca", &mut manifest_opts.ca, Some(to));
      }
      if let Some(to) = cli_opts.http_key {
        log_override("http.key", &mut manifest_opts.key, Some(to));
      }
      manifest_opts
    } else {
      HttpConfig {
        enabled: cli_opts.http_enabled,
        port: cli_opts.http_port,
        address: cli_opts.http_address,
        pem: cli_opts.http_pem,
        key: cli_opts.http_key,
        ca: cli_opts.http_ca,
      }
    };
    host_config.http = Some(http_opts);

    let lattice_opts = if let Some(mut manifest_opts) = def.host.lattice {
      if !manifest_opts.enabled {
        log_override(
          "lattice.enabled",
          &mut manifest_opts.enabled,
          cli_opts.lattice.lattice_enabled,
        );
      }
      if let Some(to) = cli_opts.lattice.nats_url {
        log_override("lattice.address", &mut manifest_opts.address, to);
      }
      if let Some(to) = cli_opts.lattice.nats_credsfile {
        log_override("lattice.creds_path", &mut manifest_opts.creds_path, Some(to));
      }
      if let Some(to) = cli_opts.lattice.nats_token {
        debug!("Overriding manifest value for 'host.lattice.token'");
        manifest_opts.token = Some(to);
      }
      Some(manifest_opts)
    } else if let Some(url) = cli_opts.lattice.nats_url {
      Some(LatticeConfig {
        enabled: cli_opts.lattice.lattice_enabled,
        address: url,
        creds_path: cli_opts.lattice.nats_credsfile,
        token: cli_opts.lattice.nats_token,
      })
    } else {
      None
    };
    host_config.lattice = lattice_opts;
  }

  HostDefinition {
    source: def.source,
    network: def.network,
    host: host_config,
    default_schematic: def.default_schematic,
  }
}

fn log_override<T: std::fmt::Debug>(field: &str, from: &mut T, to: T) {
  debug!(%field, ?from, ?to, "overriding manifest value");
  *from = to;
}
