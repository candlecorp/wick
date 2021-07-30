use vino_host::HostDefinition;

use crate::commands::HostOptions;

#[must_use]
pub(crate) fn merge_runconfig(base: HostDefinition, host: HostOptions) -> HostDefinition {
  HostDefinition {
    network: base.network,
    config: vino_host::host_definition::CommonConfiguration {
      allow_latest: host.allow_latest.unwrap_or(base.config.allow_latest),
      insecure_registries: vec![base.config.insecure_registries, host.insecure_registries].concat(),
    },
    default_schematic: base.default_schematic,
  }
}
