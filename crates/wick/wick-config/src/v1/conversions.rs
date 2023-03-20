use std::collections::HashMap;
use std::time::Duration;

use flow_expression_parser::parse_id;

use super::ComponentMetadata;
use crate::app_config::BoundResource;
use crate::component_definition::ComponentOperationExpression;
use crate::error::ManifestError;
use crate::flow_definition::{PortReference, SenderData};
use crate::host_definition::{HostConfig, MeshConfig};
use crate::utils::{opt_str_to_ipv4addr, opt_str_to_pathbuf};
use crate::{
  parse_default,
  v1,
  AppConfiguration,
  BoundComponent,
  CliConfig,
  ComponentConfiguration,
  ComponentDefinition,
  ComponentReference,
  ConnectionDefinition,
  ConnectionTargetDefinition,
  FlowOperation,
  GrpcUrlComponent,
  HttpConfig,
  HttpRouterConfig,
  HttpTriggerConfig,
  InstanceReference,
  ManifestComponent,
  Permissions,
  RawRouterConfig,
  ResourceDefinition,
  RestRouterConfig,
  Result,
  TcpPort,
  TriggerDefinition,
  UdpPort,
  WasmComponent,
};

impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      format: def.format,
      types: def.types,
      version: def.metadata.unwrap_or(ComponentMetadata::default()).version,
      host: def.host.try_into()?,
      name: def.name,
      import: def.import.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      labels: def.labels,
      operations: def
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl From<v1::ComponentOperationExpression> for ComponentOperationExpression {
  fn from(literal: v1::ComponentOperationExpression) -> Self {
    Self {
      operation: literal.operation,
      component: literal.component.into(),
    }
  }
}

impl TryFrom<v1::AppConfiguration> for AppConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::AppConfiguration) -> Result<Self> {
    Ok(AppConfiguration {
      source: None,
      format: def.format,
      version: def.metadata.unwrap_or_default().version,
      name: def.name,
      import: def.import.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      resources: def.resources.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      triggers: def.triggers.into_iter().map(|v| v.into()).collect(),
    })
  }
}

impl TryFrom<crate::v1::OperationDefinition> for FlowOperation {
  type Error = ManifestError;

  fn try_from(op: crate::v1::OperationDefinition) -> Result<Self> {
    let instances: Result<HashMap<String, InstanceReference>> = op
      .instances
      .iter()
      .map(|(key, val)| Ok((key.clone(), val.try_into()?)))
      .collect();
    let connections: Result<Vec<ConnectionDefinition>> = op.flow.iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: op.name,
      inputs: op.inputs,
      outputs: op.outputs,
      instances: instances?,
      connections: connections?,
      collections: op.components,
      constraints: Default::default(),
    })
  }
}

impl From<crate::v1::ComponentDefinition> for ComponentDefinition {
  fn from(def: crate::v1::ComponentDefinition) -> Self {
    match def {
      crate::v1::ComponentDefinition::WasmComponent(v) => ComponentDefinition::Wasm(WasmComponent {
        reference: v.reference,
        config: v.config,
        permissions: v.permissions.into(),
      }),
      crate::v1::ComponentDefinition::GrpcUrlComponent(v) => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: v.url,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ManifestComponent(v) => ComponentDefinition::Manifest(ManifestComponent {
        reference: v.reference,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ComponentReference(v) => {
        ComponentDefinition::Reference(ComponentReference { id: v.id })
      }
    }
  }
}

impl From<v1::Permissions> for Permissions {
  fn from(def: crate::v1::Permissions) -> Self {
    Self { dirs: def.dirs }
  }
}

impl TryFrom<crate::v1::InstanceDefinition> for InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v1::InstanceDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(InstanceReference {
      namespace: ns.to_owned(),
      name: name.to_owned(),
      data: def.config,
    })
  }
}

impl TryFrom<&crate::v1::InstanceDefinition> for InstanceReference {
  type Error = ManifestError;
  fn try_from(def: &crate::v1::InstanceDefinition) -> Result<Self> {
    let (ns, name) = parse_id(&def.id)?;
    Ok(InstanceReference {
      namespace: ns.to_owned(),
      name: name.to_owned(),
      data: def.config.clone(),
    })
  }
}

impl TryFrom<&crate::v1::ConnectionDefinition> for ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(def: &crate::v1::ConnectionDefinition) -> Result<Self> {
    let from: ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: ConnectionTargetDefinition = def.to.clone().try_into()?;
    let default = match &def.default {
      Some(json_str) => Some(
        parse_default(json_str).map_err(|e| ManifestError::DefaultsError(from.clone(), to.clone(), e.to_string()))?,
      ),
      None => None,
    };
    Ok(ConnectionDefinition { from, to, default })
  }
}

impl TryFrom<crate::v1::HostConfig> for HostConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: Duration::from_millis(def.timeout),
      id: def.id,
      mesh: def.mesh.and_then(|v| v.try_into().ok()),
      rpc: def.rpc.and_then(|v| v.try_into().ok()),
    })
  }
}

impl TryFrom<crate::v1::MeshConfig> for MeshConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::MeshConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      address: def.address,
      creds_path: opt_str_to_pathbuf(&def.creds_path)?,
      token: def.token,
    })
  }
}

impl TryFrom<crate::v1::HttpConfig> for HttpConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::HttpConfig) -> Result<Self> {
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

impl TryFrom<crate::v1::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(def: crate::v1::ConnectionTargetDefinition) -> Result<Self> {
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

impl From<v1::ResourceDefinition> for ResourceDefinition {
  fn from(value: v1::ResourceDefinition) -> Self {
    match value {
      v1::ResourceDefinition::TcpPort(v) => Self::TcpPort(v.into()),
      v1::ResourceDefinition::UdpPort(v) => Self::UdpPort(v.into()),
    }
  }
}
impl From<v1::TcpPort> for TcpPort {
  fn from(value: v1::TcpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}
impl From<v1::UdpPort> for UdpPort {
  fn from(value: v1::UdpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}

impl From<v1::TriggerDefinition> for TriggerDefinition {
  fn from(trigger: v1::TriggerDefinition) -> Self {
    match trigger {
      v1::TriggerDefinition::CliTrigger(cli) => Self::Cli(CliConfig {
        operation: cli.operation.into(),
        app: cli.app.map(|v| v.into()),
      }),
      v1::TriggerDefinition::HttpTrigger(v) => Self::Http(HttpTriggerConfig {
        resource: v.resource,
        routers: v.routers.into_iter().map(|v| v.into()).collect(),
      }),
    }
  }
}
impl From<v1::HttpRouter> for HttpRouterConfig {
  fn from(router: v1::HttpRouter) -> Self {
    match router {
      v1::HttpRouter::RawRouter(v) => Self::RawRouter(RawRouterConfig {
        path: v.path,
        operation: v.operation.into(),
      }),
      v1::HttpRouter::RestRouter(v) => Self::RestRouter(RestRouterConfig {
        path: v.path,
        component: v.component.into(),
      }),
    }
  }
}

impl From<v1::ComponentBinding> for BoundComponent {
  fn from(value: v1::ComponentBinding) -> Self {
    Self {
      id: value.name,
      kind: value.component.into(),
    }
  }
}

impl From<v1::ResourceBinding> for BoundResource {
  fn from(value: v1::ResourceBinding) -> Self {
    Self {
      id: value.name,
      kind: value.resource.into(),
    }
  }
}
