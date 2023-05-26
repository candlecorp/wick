use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use flow_expression_parser::ast::{self, InstancePort, InstanceTarget};
use flow_expression_parser::parse_id;
use option_utils::OptionUtils;
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
      .into_iter()
      .map(|val| Ok((val.name.clone(), val.try_into()?)))
      .collect();
    let composite = config::CompositeComponentImplementation { operations: flows? };
    Ok(config::ComponentConfiguration {
      source: None,
      types: Default::default(),
      requires: Default::default(),
      import: def
        .network
        .collections
        .into_iter()
        .map(|val| {
          Ok((
            val.namespace.clone(),
            config::ImportBinding::new(val.namespace.clone(), val.try_into()?),
          ))
        })
        .collect::<Result<HashMap<_, _>>>()?,
      component: config::ComponentImplementation::Composite(composite),
      host: def.host.try_map_into()?,
      name: def.network.name,
      tests: Vec::new(),
      labels: def.network.labels,
      metadata: None,
      resources: Default::default(),
      cached_types: Default::default(),
      type_cache: Default::default(),
      package: Default::default(),
    })
  }
}

impl TryFrom<crate::v0::CollectionDefinition> for config::ImportDefinition {
  type Error = crate::Error;
  fn try_from(def: crate::v0::CollectionDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(config::ImportDefinition::Component(def.try_into()?))
  }
}

impl TryFrom<crate::v0::CollectionDefinition> for config::ComponentDefinition {
  type Error = crate::Error;
  fn try_from(def: crate::v0::CollectionDefinition) -> std::result::Result<Self, Self::Error> {
    let kind = match def.kind {
      crate::v0::CollectionKind::Native => panic!("Can not define native components in a manifest"),
      crate::v0::CollectionKind::GrpcUrl => {
        config::ComponentDefinition::GrpcUrl(config::components::GrpcUrlComponent {
          url: def.reference.clone(),
          config: def.data.map(Into::into),
        })
      }
      #[allow(deprecated)]
      crate::v0::CollectionKind::WaPC => config::ComponentDefinition::Wasm(config::components::WasmComponent {
        reference: def.reference.clone().try_into()?,
        permissions: json_struct_to_permissions(def.data.as_ref().and_then(|v| v.get("wasi").cloned()))?,
        config: def.data.map(Into::into),
        provide: Default::default(),
      }),
      crate::v0::CollectionKind::Network => {
        config::ComponentDefinition::Manifest(config::components::ManifestComponent {
          reference: def.reference.clone().try_into()?,
          config: def.data.map(Into::into),
          provide: Default::default(),
        })
      }
    };
    Ok(kind)
  }
}

fn json_struct_to_permissions(json_perms: Option<Value>) -> Result<config::components::Permissions> {
  let perms = if let Some(json_perms) = json_perms {
    serde_json::from_value(json_perms).map_err(crate::Error::Invalid)?
  } else {
    config::components::Permissions::default()
  };

  Ok(perms)
}

impl TryFrom<crate::v0::SchematicManifest> for config::FlowOperation {
  type Error = ManifestError;

  fn try_from(manifest: crate::v0::SchematicManifest) -> Result<Self> {
    let instances: Result<HashMap<String, config::InstanceReference>> = manifest
      .instances
      .into_iter()
      .map(|(key, val)| Ok((key, val.try_into()?)))
      .collect();
    let connections: Result<Vec<ast::FlowExpression>> = manifest
      .connections
      .into_iter()
      .map(|def| Ok(ast::FlowExpression::connection(def.try_into()?)))
      .collect();
    Ok(Self {
      name: manifest.name.clone(),
      instances: instances?,
      expressions: connections?,
      components: manifest.collections,
      inputs: Default::default(),
      outputs: Default::default(),
      config: Default::default(),
      flows: Default::default(),
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
      data: def.data.map(Into::into),
    })
  }
}

impl TryFrom<crate::v0::ConnectionDefinition> for ast::ConnectionExpression {
  type Error = ManifestError;

  fn try_from(def: crate::v0::ConnectionDefinition) -> Result<Self> {
    let from: ast::ConnectionTargetExpression = def.from.clone().try_into()?;
    let to: ast::ConnectionTargetExpression = def.to.try_into()?;
    Ok(ast::ConnectionExpression::new(from, to))
  }
}

impl TryFrom<crate::v0::ConnectionTargetDefinition> for ast::ConnectionTargetExpression {
  type Error = ManifestError;

  fn try_from(def: crate::v0::ConnectionTargetDefinition) -> Result<Self> {
    Ok(ast::ConnectionTargetExpression::new_default(
      InstanceTarget::from_str(&def.instance)?,
      InstancePort::named(def.port),
      def.data,
    ))
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
