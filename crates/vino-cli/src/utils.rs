use logger::LoggingOptions;
use vino_host::HostDefinition;

use crate::commands::{
  HostOptions,
  NatsOptions,
};

fn this_or_that_option<T>(a: Option<T>, b: Option<T>) -> Option<T> {
  if a.is_some() {
    a
  } else {
    b
  }
}

#[must_use]
pub fn merge_runconfig(
  base: HostDefinition,
  nats: NatsOptions,
  host: HostOptions,
) -> HostDefinition {
  HostDefinition {
    network: base.network,
    config: vino_host::host_definition::CommonConfiguration {
      rpc_host: nats.rpc_host.unwrap_or(base.config.rpc_host),
      rpc_port: nats.rpc_port.unwrap_or(base.config.rpc_port),
      rpc_credsfile: this_or_that_option(nats.rpc_credsfile, base.config.rpc_credsfile),
      rpc_jwt: this_or_that_option(nats.rpc_jwt, base.config.rpc_jwt),
      rpc_seed: this_or_that_option(nats.rpc_seed, base.config.rpc_seed),
      control_host: nats.control_host.unwrap_or(base.config.control_host),
      control_port: nats.control_port.unwrap_or(base.config.control_port),
      control_credsfile: this_or_that_option(nats.control_credsfile, base.config.control_credsfile),
      control_jwt: this_or_that_option(nats.control_jwt, base.config.control_jwt),
      control_seed: this_or_that_option(nats.control_seed, base.config.control_seed),
      allow_oci_latest: host
        .allow_oci_latest
        .unwrap_or(base.config.allow_oci_latest),
      allowed_insecure: vec![base.config.allowed_insecure, host.allowed_insecure].concat(),
    },
    default_schematic: base.default_schematic,
  }
}

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::Logger::init(opts)?;
  Ok(())
}
