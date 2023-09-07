use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use futures::StreamExt;
use liquid_json::LiquidJsonValue;
use serde_json::Value;
use tracing::{Instrument, Span};
use wick_component_cli::options::DefaultCliOptions;
use wick_config::config::{ComponentConfiguration, ConfigurationTreeNode, HttpConfigBuilder, LiquidJsonConfig};
use wick_config::{AssetReference, WickConfiguration};
use wick_oci_utils::{OciOptions, OnExisting};
use wick_packet::{InherentData, Packet, PacketStream, RuntimeConfig};
use wick_settings::Credential;

pub(crate) async fn fetch_wick_config(
  path: &str,
  fetch_opts: OciOptions,
  runtime_config: Option<RuntimeConfig>,
  span: Span,
) -> Result<WickConfiguration> {
  let mut builder = WickConfiguration::fetch(path, fetch_opts.clone())
    .instrument(span.clone())
    .await?;

  builder
    .set_root_config(runtime_config)
    .set_env(Some(std::env::vars().collect()));
  Ok(builder.finish()?)
}

pub(crate) async fn fetch_wick_tree(
  path: &str,
  fetch_opts: OciOptions,
  runtime_config: Option<RuntimeConfig>,
  span: Span,
) -> Result<ConfigurationTreeNode<WickConfiguration>> {
  let env: HashMap<String, String> = std::env::vars().collect();
  let config = WickConfiguration::fetch_tree(path, runtime_config, Some(env), fetch_opts.clone())
    .instrument(span.clone())
    .await?;

  Ok(config)
}

pub(crate) fn merge_config(
  def: ComponentConfiguration,
  local_cli_opts: &crate::options::oci::OciOptions,
  server_cli_opts: Option<DefaultCliOptions>,
) -> ComponentConfiguration {
  let mut merged_manifest = def;
  let mut host_config = merged_manifest.host().cloned().unwrap_or_default();

  host_config.set_allow_latest(local_cli_opts.allow_latest || host_config.allow_latest());
  host_config.set_insecure_registries(
    vec![
      host_config.insecure_registries().to_vec(),
      local_cli_opts.insecure_registries.clone(),
    ]
    .concat(),
  );

  if let Some(cli_opts) = server_cli_opts {
    #[allow(clippy::option_if_let_else)]
    if let Some(manifest_opts) = host_config.rpc_mut().as_mut() {
      if !manifest_opts.enabled() {
        log_override("rpc.enabled", manifest_opts.enabled_mut(), cli_opts.rpc_enabled);
      }
      if let Some(to) = cli_opts.rpc_address {
        log_override("rpc.address", manifest_opts.address_mut(), Some(to));
      }
      if let Some(to) = cli_opts.rpc_port {
        log_override("rpc.port", manifest_opts.port_mut(), Some(to));
      }
      if let Some(to) = cli_opts.rpc_pem {
        log_override("rpc.pem", manifest_opts.pem_mut(), Some(AssetReference::new(to)));
      }
      if let Some(to) = cli_opts.rpc_ca {
        log_override("rpc.ca", manifest_opts.ca_mut(), Some(AssetReference::new(to)));
      }
      if let Some(to) = cli_opts.rpc_key {
        log_override("rpc.key", manifest_opts.key_mut(), Some(AssetReference::new(to)));
      }
    } else {
      host_config.set_rpc(
        HttpConfigBuilder::default()
          .enabled(cli_opts.rpc_enabled)
          .port(cli_opts.rpc_port)
          .address(cli_opts.rpc_address)
          .pem(cli_opts.rpc_pem.map(AssetReference::new))
          .key(cli_opts.rpc_key.map(AssetReference::new))
          .ca(cli_opts.rpc_ca.map(AssetReference::new))
          .build()
          .unwrap(),
      );
    };
  }
  merged_manifest.set_host(Some(host_config));

  merged_manifest
}

pub(crate) fn get_auth_for_scope(
  configured_creds: Option<&Credential>,
  override_username: Option<&str>,
  override_password: Option<&str>,
) -> (Option<String>, Option<String>) {
  let mut username = None;
  let mut password = None;

  if let Some(creds) = configured_creds {
    match &creds.auth {
      wick_settings::Auth::Basic(basic) => {
        debug!("using basic auth from configuration settings.");
        username = Some(basic.username.clone());
        password = Some(basic.password.clone());
      }
    };
  }

  if override_username.is_some() {
    debug!("overriding username from arguments.");
    username = override_username.map(|v| v.to_owned());
  }

  if override_password.is_some() {
    debug!("overriding password from arguments.");
    password = override_password.map(|v| v.to_owned());
  }

  (username, password)
}

pub(crate) fn reconcile_fetch_options(
  reference: &str,
  settings: &wick_settings::Settings,
  opts: crate::options::oci::OciOptions,
  output: Option<PathBuf>,
) -> OciOptions {
  let xdg = wick_xdg::Settings::new();
  let configured_creds = settings.credentials.iter().find(|c| reference.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(configured_creds, opts.username.as_deref(), opts.password.as_deref());

  let mut oci_opts = OciOptions::default();
  oci_opts
    .set_allow_insecure(opts.insecure_registries)
    .set_allow_latest(true)
    .set_username(username)
    .set_password(password)
    .set_on_existing(if opts.force {
      OnExisting::Overwrite
    } else {
      OnExisting::Ignore
    });

  if let Some(output) = output {
    oci_opts.set_cache_dir(output);
  } else {
    // otherwise, use the global cache.
    oci_opts.set_cache_dir(xdg.global().cache().clone());
  };
  oci_opts
}

fn log_override<T: std::fmt::Debug>(field: &str, from: &mut T, to: T) {
  debug!(%field, ?from, ?to, "overriding manifest value");
  *from = to;
}

pub(crate) async fn print_stream_json(
  mut stream: PacketStream,

  filter: &[String],
  _terse: bool,
  raw: bool,
) -> Result<()> {
  if !filter.is_empty() {
    trace!(?filter, "cli:output:filter");
  }
  while let Some(packet) = stream.next().await {
    match packet {
      Ok(packet) => {
        if (packet.is_done()) && !raw {
          continue;
        }
        if !filter.is_empty() && !filter.iter().any(|name| name == packet.port()) {
          tracing::debug!(port = %packet.port(), "cli:output:filtering");
          continue;
        }
        let json = packet.to_json();
        println!("{}", json);
      }
      Err(e) => {
        error!(error = %e, "cli:output:error");
        let packet = Packet::component_error(e.to_string());
        println!("{}", packet.to_json());
      }
    }
  }
  trace!("cli:output:complete");
  Ok(())
}

pub(crate) fn parse_config_string(source: Option<&str>) -> Result<Option<RuntimeConfig>> {
  let component_config = match source {
    Some(c) => {
      let config = serde_json::from_str::<LiquidJsonValue>(c)
        .map_err(|e| anyhow::anyhow!("Failed to parse config argument as JSON: {}", e))?;
      let ctx = LiquidJsonConfig::make_context(
        None,
        None,
        None,
        Some(&std::env::vars().collect::<HashMap<_, _>>()),
        Some(&InherentData::unsafe_default()),
      )?;
      let rendered = RuntimeConfig::from_value(config.render(&ctx)?)
        .map_err(|_| anyhow::anyhow!("configuration could not be parsed as an object"))?;

      trace!(config=?rendered, "rendered config");
      Some(rendered)
    }
    None => None,
  };
  Ok(component_config)
}

pub(crate) fn packet_from_kv_json(values: &[String]) -> Result<Vec<Packet>> {
  let mut packets = Vec::new();

  for input in values {
    match input.split_once('=') {
      Some((port, value)) => {
        debug!(port, value, "cli:args:port-data");
        let val: Value = serde_json::from_str(value).map_err(|e| anyhow!("Could not parse JSON into a packet: {e}"))?;
        packets.push(Packet::encode(port, val));
      }
      None => bail!("Invalid port=value pair: '{input}'"),
    }
  }
  Ok(packets)
}
