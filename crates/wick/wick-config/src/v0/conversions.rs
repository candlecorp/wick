use std::collections::HashMap;
use std::time::Duration;

use flow_expression_parser::parse_id;
use serde_json::Value;

use crate::component_config::{ComponentImplementation, CompositeComponentConfiguration};
use crate::component_definition::{GrpcUrlComponent, NativeComponent};
use crate::config::ComponentConfiguration;
use crate::error::ManifestError;
use crate::flow_definition::{PortReference, SenderData};
use crate::host_definition::HostConfig;
use crate::utils::{opt_str_to_ipv4addr, opt_str_to_pathbuf};
use crate::{
  v0,
  BoundComponent,
  ComponentDefinition,
  ConnectionDefinition,
  ConnectionTargetDefinition,
  FlowOperation,
  HttpConfig,
  InstanceReference,
  ManifestComponent,
  Permissions,
  Result,
  WasmComponent,
};

impl TryFrom<v0::HostManifest> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v0::HostManifest) -> Result<Self> {
    let flows: Result<HashMap<String, FlowOperation>> = def
      .network
      .schematics
      .iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    let composite = CompositeComponentConfiguration {
      types: Default::default(),
      import: def
        .network
        .collections
        .iter()
        .map(|val| {
          Ok((
            val.namespace.clone(),
            BoundComponent::new(val.namespace.clone(), val.try_into()?),
          ))
        })
        .collect::<Result<HashMap<_, _>>>()?,
      operations: flows?,
    };
    Ok(ComponentConfiguration {
      source: None,
      format: def.format,
      version: def.version,
      component: ComponentImplementation::Composite(composite),
      host: def.host.try_into()?,
      name: def.network.name,
      tests: Vec::new(),
      labels: def.network.labels,
    })
  }
}

impl TryFrom<&crate::v0::CollectionDefinition> for ComponentDefinition {
  type Error = crate::Error;
  fn try_from(def: &crate::v0::CollectionDefinition) -> std::result::Result<Self, Self::Error> {
    let kind = match def.kind {
      crate::v0::CollectionKind::Native => ComponentDefinition::Native(NativeComponent {}),
      crate::v0::CollectionKind::GrpcUrl => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: def.reference.clone(),
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::WaPC => ComponentDefinition::Wasm(WasmComponent {
        reference: def.reference.clone(),
        permissions: json_struct_to_permissions(def.data.get("wasi"))?,
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Network => ComponentDefinition::Manifest(ManifestComponent {
        reference: def.reference.clone(),
        config: def.data.clone(),
      }),
    };
    Ok(kind)
  }
}

fn json_struct_to_permissions(json_perms: Option<&Value>) -> Result<Permissions> {
  let perms = if let Some(json_perms) = json_perms {
    serde_json::from_value(json_perms.clone()).map_err(crate::Error::Invalid)?
  } else {
    Permissions::default()
  };

  Ok(perms)
}

impl TryFrom<&crate::v0::SchematicManifest> for FlowOperation {
  type Error = ManifestError;

  fn try_from(manifest: &crate::v0::SchematicManifest) -> Result<Self> {
    let instances: Result<HashMap<String, InstanceReference>> = manifest
      .instances
      .iter()
      .map(|(key, val)| Ok((key.clone(), val.try_into()?)))
      .collect();
    let connections: Result<Vec<ConnectionDefinition>> =
      manifest.connections.iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: manifest.name.clone(),
      inputs: Default::default(),
      outputs: Default::default(),
      instances: instances?,
      connections: connections?,
      components: manifest.collections.clone(),
    })
  }
}

impl TryFrom<crate::v0::ComponentDefinition> for InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(InstanceReference {
      component_id: ns.to_owned(),
      name: name.to_owned(),
      data: def.data,
    })
  }
}

impl TryFrom<&crate::v0::ComponentDefinition> for InstanceReference {
  type Error = ManifestError;
  fn try_from(def: &crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(InstanceReference {
      component_id: ns.to_owned(),
      name: name.to_owned(),
      data: def.data.clone(),
    })
  }
}

impl TryFrom<&crate::v0::ConnectionDefinition> for ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(def: &crate::v0::ConnectionDefinition) -> Result<Self> {
    let from: ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: ConnectionTargetDefinition = def.to.clone().try_into()?;
    Ok(ConnectionDefinition { from, to })
  }
}

impl TryFrom<crate::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(def: crate::v0::ConnectionTargetDefinition) -> Result<Self> {
    let data = def.data.map(|json| SenderData { inner: json });
    Ok(ConnectionTargetDefinition {
      target: PortReference {
        instance: def.instance,
        port: def.port,
      },
      data,
    })
  }
}

impl TryFrom<crate::v0::HostConfig> for HostConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: Duration::from_millis(def.timeout),
      rpc: def.rpc.and_then(|v| v.try_into().ok()),
    })
  }
}

impl TryFrom<crate::v0::HttpConfig> for HttpConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::HttpConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      port: def.port,
      address: opt_str_to_ipv4addr(&def.address)?,
      pem: opt_str_to_pathbuf(&def.pem)?,
      key: opt_str_to_pathbuf(&def.key)?,
      ca: opt_str_to_pathbuf(&def.ca)?,
    })
  }
}
