use std::collections::HashMap;
use std::time::Duration;

use flow_expression_parser::parse_id;
use serde_json::Value;

use crate::error::ManifestError;
use crate::utils::opt_str_to_ipv4addr;
use crate::{config, v0, Result};

impl TryFrom<v0::HostManifest> for config::ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v0::HostManifest) -> Result<Self> {
    let flows: Result<HashMap<String, config::FlowOperation>> = def
      .network
      .schematics
      .iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    let composite = config::CompositeComponentConfiguration {
      types: Default::default(),
      import: def
        .network
        .collections
        .iter()
        .map(|val| {
          Ok((
            val.namespace.clone(),
            config::BoundComponent::new(val.namespace.clone(), val.try_into()?),
          ))
        })
        .collect::<Result<HashMap<_, _>>>()?,
      operations: flows?,
    };
    Ok(config::ComponentConfiguration {
      source: None,
      component: config::ComponentImplementation::Composite(composite),
      host: def.host.try_into()?,
      name: def.network.name,
      tests: Vec::new(),
      labels: def.network.labels,
      metadata: None,
    })
  }
}

impl TryFrom<&crate::v0::CollectionDefinition> for config::ComponentDefinition {
  type Error = crate::Error;
  fn try_from(def: &crate::v0::CollectionDefinition) -> std::result::Result<Self, Self::Error> {
    let kind = match def.kind {
      crate::v0::CollectionKind::Native => config::ComponentDefinition::Native(config::NativeComponent {}),
      crate::v0::CollectionKind::GrpcUrl => config::ComponentDefinition::GrpcUrl(config::GrpcUrlComponent {
        url: def.reference.clone(),
        config: def.data.clone(),
      }),
      #[allow(deprecated)]
      crate::v0::CollectionKind::WaPC => config::ComponentDefinition::Wasm(config::WasmComponent {
        reference: def.reference.clone().try_into()?,
        permissions: json_struct_to_permissions(def.data.get("wasi"))?,
        config: def.data.clone(),
      }),
      crate::v0::CollectionKind::Network => config::ComponentDefinition::Manifest(config::ManifestComponent {
        reference: def.reference.clone().try_into()?,
        config: def.data.clone(),
      }),
    };
    Ok(kind)
  }
}

fn json_struct_to_permissions(json_perms: Option<&Value>) -> Result<config::Permissions> {
  let perms = if let Some(json_perms) = json_perms {
    serde_json::from_value(json_perms.clone()).map_err(crate::Error::Invalid)?
  } else {
    config::Permissions::default()
  };

  Ok(perms)
}

impl TryFrom<&crate::v0::SchematicManifest> for config::FlowOperation {
  type Error = ManifestError;

  fn try_from(manifest: &crate::v0::SchematicManifest) -> Result<Self> {
    let instances: Result<HashMap<String, config::InstanceReference>> = manifest
      .instances
      .iter()
      .map(|(key, val)| Ok((key.clone(), val.try_into()?)))
      .collect();
    let connections: Result<Vec<config::ConnectionDefinition>> =
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

impl TryFrom<crate::v0::ComponentDefinition> for config::InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(config::InstanceReference {
      component_id: ns.to_owned(),
      name: name.to_owned(),
      data: def.data,
    })
  }
}

impl TryFrom<&crate::v0::ComponentDefinition> for config::InstanceReference {
  type Error = ManifestError;
  fn try_from(def: &crate::v0::ComponentDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(config::InstanceReference {
      component_id: ns.to_owned(),
      name: name.to_owned(),
      data: def.data.clone(),
    })
  }
}

impl TryFrom<&crate::v0::ConnectionDefinition> for config::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(def: &crate::v0::ConnectionDefinition) -> Result<Self> {
    let from: config::ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: config::ConnectionTargetDefinition = def.to.clone().try_into()?;
    Ok(config::ConnectionDefinition { from, to })
  }
}

impl TryFrom<crate::v0::ConnectionTargetDefinition> for config::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(def: crate::v0::ConnectionTargetDefinition) -> Result<Self> {
    let data = def.data.map(|json| config::SenderData { inner: json });
    Ok(config::ConnectionTargetDefinition {
      target: config::PortReference {
        instance: def.instance,
        port: def.port,
      },
      data,
    })
  }
}

impl TryFrom<crate::v0::HostConfig> for config::HostConfig {
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

impl TryFrom<crate::v0::HttpConfig> for config::HttpConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v0::HttpConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      port: def.port,
      address: opt_str_to_ipv4addr(&def.address)?,
      pem: match def.pem {
        Some(v) => Some(v.try_into()?),
        None => None,
      },
      key: match def.key {
        Some(v) => Some(v.try_into()?),
        None => None,
      },
      ca: match def.ca {
        Some(v) => Some(v.try_into()?),
        None => None,
      },
    })
  }
}
