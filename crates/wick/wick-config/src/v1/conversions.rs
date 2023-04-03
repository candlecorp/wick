use std::collections::HashMap;
use std::time::Duration;

// use flow_expression_parser::parse_id;
use crate::app_config::{
  AppConfiguration,
  BoundResource,
  CliConfig,
  HttpRouterConfig,
  HttpTriggerConfig,
  RawRouterConfig,
  ResourceDefinition,
  RestRouterConfig,
  TcpPort,
  TriggerDefinition,
  UdpPort,
};
use crate::component_config::{
  ComponentImplementation,
  CompositeComponentConfiguration,
  OperationSignature,
  WasmComponentConfiguration,
};
use crate::config::common::component_definition::{
  BoundComponent,
  ComponentDefinition,
  ComponentOperationExpression,
  ComponentReference,
  GrpcUrlComponent,
  ManifestComponent,
};
use crate::config::common::flow_definition::{PortReference, SenderData};
use crate::config::common::host_definition::HostConfig;
use crate::config::common::test_case;
use crate::config::{self, test_config, types_config, ComponentConfiguration};
use crate::error::ManifestError;
use crate::utils::opt_str_to_ipv4addr;
use crate::{v1, Result, WickConfiguration};

impl TryFrom<v1::WickConfig> for WickConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::WickConfig) -> Result<Self> {
    let new = match def {
      v1::WickConfig::AppConfiguration(v) => WickConfiguration::App(v.try_into()?),
      v1::WickConfig::ComponentConfiguration(v) => WickConfiguration::Component(v.try_into()?),
      v1::WickConfig::TypesConfiguration(v) => WickConfiguration::Types(v.try_into()?),
      v1::WickConfig::TestConfiguration(v) => WickConfiguration::Tests(v.try_into()?),
    };
    Ok(new)
  }
}

impl TryFrom<v1::TestConfiguration> for test_config::TestConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::TestConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      tests: value.tests.into_iter().map(|v| v.into()).collect(),
      source: None,
    })
  }
}
impl TryFrom<v1::TypesConfiguration> for types_config::TypesConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::TypesConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      types: value.types,
      source: None,
    })
  }
}
impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      metadata: def.metadata.map(TryInto::try_into).transpose()?,
      host: def.host.try_into()?,
      name: def.name,
      labels: def.labels,
      tests: def.tests.into_iter().map(|v| v.into()).collect(),
      component: def.component.try_into()?,
    })
  }
}

impl TryFrom<ComponentConfiguration> for v1::ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: ComponentConfiguration) -> Result<Self> {
    Ok(v1::ComponentConfiguration {
      metadata: def.metadata.map(TryInto::try_into).transpose()?,
      host: def.host.try_into()?,
      name: def.name,
      labels: def.labels,
      tests: def.tests.into_iter().map(|v| v.into()).collect(),
      component: def.component.try_into()?,
    })
  }
}

impl TryFrom<v1::WasmComponentConfiguration> for WasmComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: v1::WasmComponentConfiguration) -> Result<Self> {
    Ok(Self {
      reference: value.reference.try_into()?,
      operations: value
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
      types: value.types,
    })
  }
}

impl TryFrom<v1::CompositeComponentConfiguration> for CompositeComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: v1::CompositeComponentConfiguration) -> Result<Self> {
    Ok(Self {
      operations: value
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
      types: value.types,
      import: value
        .import
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<CompositeComponentConfiguration> for v1::CompositeComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: CompositeComponentConfiguration) -> Result<Self> {
    Ok(Self {
      operations: value
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
      types: value.types,
      import: value
        .import
        .into_values()
        .map(|v| v.try_into())
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<WasmComponentConfiguration> for v1::WasmComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: WasmComponentConfiguration) -> Result<Self> {
    Ok(Self {
      operations: value
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
      types: value.types,
      reference: value.reference.try_into()?,
    })
  }
}

impl TryFrom<v1::ComponentOperationExpression> for ComponentOperationExpression {
  type Error = ManifestError;
  fn try_from(literal: v1::ComponentOperationExpression) -> Result<Self> {
    Ok(Self {
      operation: literal.name,
      component: literal.component.try_into()?,
    })
  }
}

impl TryFrom<v1::AppConfiguration> for AppConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::AppConfiguration) -> Result<Self> {
    Ok(AppConfiguration {
      source: None,
      metadata: def.metadata.map(TryInto::try_into).transpose()?,
      name: def.name,
      host: def.host.try_into()?,
      import: def
        .import
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      resources: def.resources.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      triggers: def.triggers.into_iter().map(|v| v.try_into()).collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<AppConfiguration> for v1::AppConfiguration {
  type Error = ManifestError;

  fn try_from(value: AppConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      metadata: value.metadata.map(TryInto::try_into).transpose()?,
      name: value.name,
      import: value
        .import
        .into_values()
        .map(|v| v.try_into())
        .collect::<Result<_>>()?,
      resources: value.resources.into_values().map(|v| v.into()).collect(),
      triggers: value
        .triggers
        .into_iter()
        .map(|v| v.try_into())
        .collect::<Result<_>>()?,
      host: value.host.try_into()?,
    })
  }
}

impl TryFrom<TriggerDefinition> for v1::TriggerDefinition {
  type Error = ManifestError;
  fn try_from(value: TriggerDefinition) -> Result<Self> {
    Ok(match value {
      TriggerDefinition::Http(v) => v1::TriggerDefinition::HttpTrigger(v.try_into()?),
      TriggerDefinition::Cli(v) => v1::TriggerDefinition::CliTrigger(v.try_into()?),
    })
  }
}

impl TryFrom<CliConfig> for v1::CliTrigger {
  type Error = ManifestError;
  fn try_from(value: CliConfig) -> Result<Self> {
    Ok(Self {
      operation: value.operation.try_into()?,
      app: value.app.map(TryInto::try_into).transpose()?,
    })
  }
}

impl TryFrom<HttpTriggerConfig> for v1::HttpTrigger {
  type Error = ManifestError;
  fn try_from(value: HttpTriggerConfig) -> Result<Self> {
    Ok(Self {
      resource: value.resource,
      routers: value.routers.into_iter().map(|v| v.try_into()).collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<HttpRouterConfig> for v1::HttpRouter {
  type Error = ManifestError;
  fn try_from(value: HttpRouterConfig) -> Result<Self> {
    Ok(match value {
      HttpRouterConfig::RawRouter(v) => v1::HttpRouter::RawRouter(v.try_into()?),
      HttpRouterConfig::RestRouter(v) => v1::HttpRouter::RestRouter(v.try_into()?),
    })
  }
}

impl TryFrom<RawRouterConfig> for v1::RawRouter {
  type Error = ManifestError;
  fn try_from(value: RawRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      operation: value.operation.try_into()?,
    })
  }
}

impl TryFrom<RestRouterConfig> for v1::RestRouter {
  type Error = ManifestError;
  fn try_from(value: RestRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      component: value.component.try_into()?,
    })
  }
}

impl TryFrom<ComponentOperationExpression> for v1::ComponentOperationExpression {
  type Error = ManifestError;
  fn try_from(value: ComponentOperationExpression) -> Result<Self> {
    Ok(Self {
      name: value.operation,
      component: value.component.try_into()?,
    })
  }
}

impl From<ResourceDefinition> for v1::ResourceDefinition {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::TcpPort(v) => v1::ResourceDefinition::TcpPort(v.into()),
      ResourceDefinition::UdpPort(v) => v1::ResourceDefinition::UdpPort(v.into()),
    }
  }
}

impl From<UdpPort> for v1::UdpPort {
  fn from(value: UdpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}

impl From<TcpPort> for v1::TcpPort {
  fn from(value: TcpPort) -> Self {
    Self {
      port: value.port,
      address: value.address,
    }
  }
}

impl TryFrom<crate::v1::CompositeOperationDefinition> for config::FlowOperation {
  type Error = ManifestError;

  fn try_from(op: crate::v1::CompositeOperationDefinition) -> Result<Self> {
    let instances: Result<HashMap<String, config::InstanceReference>> = op
      .instances
      .into_iter()
      .map(|v| Ok((v.name.clone(), v.try_into()?)))
      .collect();
    let connections: Result<Vec<config::ConnectionDefinition>> = op.flow.iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: op.name,
      inputs: op.inputs,
      outputs: op.outputs,
      instances: instances?,
      connections: connections?,
      components: op.components,
    })
  }
}

impl TryFrom<v1::ComponentKind> for ComponentImplementation {
  type Error = ManifestError;
  fn try_from(value: v1::ComponentKind) -> Result<Self> {
    Ok(match value {
      v1::ComponentKind::CompositeComponentConfiguration(v) => ComponentImplementation::Composite(v.try_into()?),
      v1::ComponentKind::WasmComponentConfiguration(v) => ComponentImplementation::Wasm(v.try_into()?),
    })
  }
}

impl TryFrom<ComponentImplementation> for v1::ComponentKind {
  type Error = ManifestError;
  fn try_from(value: ComponentImplementation) -> Result<Self> {
    Ok(match value {
      ComponentImplementation::Composite(v) => v1::ComponentKind::CompositeComponentConfiguration(v.try_into()?),
      ComponentImplementation::Wasm(v) => v1::ComponentKind::WasmComponentConfiguration(v.try_into()?),
    })
  }
}

impl TryFrom<crate::v1::OperationDefinition> for OperationSignature {
  type Error = ManifestError;

  fn try_from(op: crate::v1::OperationDefinition) -> Result<Self> {
    Ok(Self {
      name: op.name,
      inputs: op.inputs,
      outputs: op.outputs,
    })
  }
}

impl TryFrom<OperationSignature> for crate::v1::OperationDefinition {
  type Error = ManifestError;

  fn try_from(op: OperationSignature) -> Result<Self> {
    Ok(Self {
      name: op.name,
      inputs: op.inputs,
      outputs: op.outputs,
    })
  }
}

impl TryFrom<BoundComponent> for v1::ComponentBinding {
  type Error = ManifestError;
  fn try_from(def: BoundComponent) -> Result<Self> {
    Ok(Self {
      name: def.id,
      component: def.kind.try_into()?,
    })
  }
}

impl From<BoundResource> for v1::ResourceBinding {
  fn from(value: BoundResource) -> Self {
    Self {
      name: value.id,
      resource: value.kind.into(),
    }
  }
}

impl TryFrom<ComponentDefinition> for v1::ComponentDefinition {
  type Error = ManifestError;
  fn try_from(kind: ComponentDefinition) -> Result<Self> {
    let def = match kind {
      #[allow(deprecated)]
      ComponentDefinition::Wasm(_) => unimplemented!(
        "Wasm component definition is no longer supported in v1 manifests. Use ManifestComponent instead."
      ),
      ComponentDefinition::GrpcUrl(grpc) => Self::GrpcUrlComponent(grpc.into()),
      ComponentDefinition::Native(_) => todo!(),
      ComponentDefinition::Reference(v) => Self::ComponentReference(v.into()),
      ComponentDefinition::Manifest(v) => Self::ManifestComponent(v.try_into()?),
    };
    Ok(def)
  }
}

impl TryFrom<ManifestComponent> for v1::ManifestComponent {
  type Error = ManifestError;
  fn try_from(def: ManifestComponent) -> Result<Self> {
    Ok(Self {
      reference: def.reference.try_into()?,
      config: def.config,
    })
  }
}

impl From<ComponentReference> for v1::ComponentReference {
  fn from(value: ComponentReference) -> Self {
    Self { id: value.id }
  }
}

impl From<GrpcUrlComponent> for v1::GrpcUrlComponent {
  fn from(def: GrpcUrlComponent) -> Self {
    Self {
      url: def.url,
      config: def.config,
    }
  }
}

impl From<config::Permissions> for v1::Permissions {
  fn from(value: config::Permissions) -> Self {
    Self { dirs: value.dirs }
  }
}

impl TryFrom<config::FlowOperation> for v1::CompositeOperationDefinition {
  type Error = ManifestError;

  fn try_from(value: config::FlowOperation) -> std::result::Result<Self, Self::Error> {
    let instances: Vec<v1::InstanceBinding> = value.instances.into_iter().map(|(id, val)| (id, val).into()).collect();
    let connections: Result<Vec<v1::ConnectionDefinition>> =
      value.connections.into_iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
      instances,
      flow: connections?,
      components: value.components,
    })
  }
}

impl TryFrom<config::ConnectionDefinition> for v1::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(value: config::ConnectionDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      from: value.from.try_into()?,
      to: value.to.try_into()?,
    })
  }
}

impl TryFrom<config::ConnectionTargetDefinition> for v1::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(value: config::ConnectionTargetDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      data: value.data.map(|v| v.inner),
      instance: value.target.instance,
      port: value.target.port,
    })
  }
}

impl From<(String, config::InstanceReference)> for v1::InstanceBinding {
  fn from(value: (String, config::InstanceReference)) -> Self {
    let id = value.0;
    let value = value.1;
    Self {
      name: id,
      operation: v1::ComponentOperationExpression {
        name: value.name,
        component: v1::ComponentDefinition::ComponentReference(v1::ComponentReference { id: value.component_id }),
      },
      config: value.data,
    }
  }
}

impl TryFrom<crate::v1::ComponentDefinition> for ComponentDefinition {
  type Error = ManifestError;
  fn try_from(def: crate::v1::ComponentDefinition) -> Result<Self> {
    let res = match def {
      crate::v1::ComponentDefinition::GrpcUrlComponent(v) => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: v.url,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ManifestComponent(v) => ComponentDefinition::Manifest(ManifestComponent {
        reference: v.reference.try_into()?,
        config: v.config,
      }),
      crate::v1::ComponentDefinition::ComponentReference(v) => {
        ComponentDefinition::Reference(ComponentReference { id: v.id })
      }
    };
    Ok(res)
  }
}

impl From<v1::Permissions> for config::Permissions {
  fn from(def: crate::v1::Permissions) -> Self {
    Self { dirs: def.dirs }
  }
}

impl TryFrom<crate::v1::InstanceBinding> for config::InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v1::InstanceBinding) -> Result<Self> {
    let ns = def.operation.component.component_id().unwrap_or("<anonymous>");
    let name = def.operation.name;
    Ok(config::InstanceReference {
      component_id: ns.to_owned(),
      name,
      data: def.config,
    })
  }
}

impl TryFrom<&crate::v1::ConnectionDefinition> for config::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(def: &crate::v1::ConnectionDefinition) -> Result<Self> {
    let from: config::ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: config::ConnectionTargetDefinition = def.to.clone().try_into()?;
    Ok(config::ConnectionDefinition { from, to })
  }
}

impl TryFrom<crate::v1::HostConfig> for HostConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: Duration::from_millis(def.timeout),
      rpc: def.rpc.and_then(|v| v.try_into().ok()),
    })
  }
}

impl TryFrom<HostConfig> for crate::v1::HostConfig {
  type Error = ManifestError;
  fn try_from(def: HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: def.timeout.as_millis() as u64,
      rpc: def.rpc.and_then(|v| v.try_into().ok()),
    })
  }
}

impl TryFrom<crate::v1::HttpConfig> for config::HttpConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::HttpConfig) -> Result<Self> {
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

impl TryFrom<config::HttpConfig> for crate::v1::HttpConfig {
  type Error = ManifestError;
  fn try_from(def: config::HttpConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      port: def.port,
      address: def.address.map(|v| v.to_string()),
      pem: def.pem.map(TryInto::try_into).transpose()?,
      key: def.key.map(TryInto::try_into).transpose()?,
      ca: def.ca.map(TryInto::try_into).transpose()?,
    })
  }
}

impl TryFrom<crate::v1::ConnectionTargetDefinition> for config::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(def: crate::v1::ConnectionTargetDefinition) -> Result<Self> {
    let data = def.data.map(|json| SenderData { inner: json });
    Ok(config::ConnectionTargetDefinition {
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

impl TryFrom<v1::TriggerDefinition> for TriggerDefinition {
  type Error = ManifestError;
  fn try_from(trigger: v1::TriggerDefinition) -> Result<Self> {
    let rv = match trigger {
      v1::TriggerDefinition::CliTrigger(cli) => Self::Cli(CliConfig {
        operation: cli.operation.try_into()?,
        app: cli.app.map(|v| v.try_into()).transpose()?,
      }),
      v1::TriggerDefinition::HttpTrigger(v) => Self::Http(HttpTriggerConfig {
        resource: v.resource,
        routers: v.routers.into_iter().map(|v| v.try_into()).collect::<Result<_>>()?,
      }),
    };
    Ok(rv)
  }
}
impl TryFrom<v1::HttpRouter> for HttpRouterConfig {
  type Error = ManifestError;
  fn try_from(router: v1::HttpRouter) -> Result<Self> {
    let rv = match router {
      v1::HttpRouter::RawRouter(v) => Self::RawRouter(RawRouterConfig {
        path: v.path,
        operation: v.operation.try_into()?,
      }),
      v1::HttpRouter::RestRouter(v) => Self::RestRouter(RestRouterConfig {
        path: v.path,
        component: v.component.try_into()?,
      }),
    };
    Ok(rv)
  }
}

impl TryFrom<v1::ComponentBinding> for BoundComponent {
  type Error = ManifestError;
  fn try_from(value: v1::ComponentBinding) -> Result<Self> {
    Ok(Self {
      id: value.name,
      kind: value.component.try_into()?,
    })
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

impl From<v1::TestDefinition> for test_case::TestCase {
  fn from(value: v1::TestDefinition) -> Self {
    Self {
      name: value.name,
      operation: value.operation,
      inputs: value.input.into_iter().map(|v| v.into()).collect(),
      outputs: value.output.into_iter().map(|v| v.into()).collect(),
      inherent: value.inherent.map(|v| v.into()),
    }
  }
}

impl From<v1::PacketData> for test_case::TestPacket {
  fn from(value: v1::PacketData) -> Self {
    match value {
      v1::PacketData::PayloadData(v) => test_case::TestPacket::PayloadData(v.into()),
      v1::PacketData::ErrorData(v) => test_case::TestPacket::ErrorData(v.into()),
    }
  }
}

impl From<v1::PayloadData> for config::SuccessPayload {
  fn from(value: v1::PayloadData) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map(|v| v.into()),
      data: value.data,
    }
  }
}

impl From<v1::ErrorData> for config::ErrorPayload {
  fn from(value: v1::ErrorData) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map(|v| v.into()),
      message: value.message,
    }
  }
}

impl From<v1::PacketFlags> for config::PacketFlags {
  fn from(value: v1::PacketFlags) -> Self {
    Self {
      done: value.done,
      open: value.open,
      close: value.close,
    }
  }
}

impl From<v1::InherentData> for config::InherentConfig {
  fn from(value: v1::InherentData) -> Self {
    Self {
      seed: value.seed,
      timestamp: value.timestamp,
    }
  }
}

impl From<test_case::TestCase> for v1::TestDefinition {
  fn from(value: test_case::TestCase) -> Self {
    Self {
      name: value.name,
      operation: value.operation,
      input: value.inputs.into_iter().map(|v| v.into()).collect(),
      output: value.outputs.into_iter().map(|v| v.into()).collect(),
      inherent: value.inherent.map(|v| v.into()),
    }
  }
}

impl From<test_case::ErrorPayload> for v1::ErrorData {
  fn from(value: test_case::ErrorPayload) -> Self {
    Self {
      name: value.port,
      flags: value.flags.map(|v| v.into()),
      message: value.message,
    }
  }
}

impl From<test_case::SuccessPayload> for v1::PayloadData {
  fn from(value: test_case::SuccessPayload) -> Self {
    Self {
      name: value.port,
      flags: value.flags.map(|v| v.into()),
      data: value.data,
    }
  }
}

impl From<test_case::PacketFlags> for v1::PacketFlags {
  fn from(value: test_case::PacketFlags) -> Self {
    Self {
      done: value.done,
      open: value.open,
      close: value.close,
    }
  }
}

impl From<test_case::InherentConfig> for v1::InherentData {
  fn from(value: test_case::InherentConfig) -> Self {
    Self {
      seed: value.seed,
      timestamp: value.timestamp,
    }
  }
}

impl From<test_case::TestPacket> for v1::PacketData {
  fn from(value: test_case::TestPacket) -> Self {
    match value {
      test_case::TestPacket::PayloadData(v) => v1::PacketData::PayloadData(v.into()),
      test_case::TestPacket::ErrorData(v) => v1::PacketData::ErrorData(v.into()),
    }
  }
}

impl TryFrom<v1::helpers::LocationReference> for config::AssetReference {
  type Error = crate::Error;
  fn try_from(value: v1::helpers::LocationReference) -> Result<Self> {
    Ok(value.0.try_into()?)
  }
}

impl TryFrom<config::AssetReference> for v1::helpers::LocationReference {
  type Error = crate::Error;
  fn try_from(value: config::AssetReference) -> Result<Self> {
    Ok(Self(value.location().to_owned()))
  }
}

impl TryFrom<v1::Metadata> for config::Metadata {
  type Error = crate::Error;
  fn try_from(value: v1::Metadata) -> Result<Self> {
    Ok(Self {
      version: value.version,
      authors: value.authors,
      vendors: value.vendors,
      description: value.description,
      documentation: value.documentation,
      licenses: value.licenses,
      icon: value.icon.map(TryInto::try_into).transpose()?,
    })
  }
}

impl TryFrom<config::Metadata> for v1::Metadata {
  type Error = crate::Error;
  fn try_from(value: config::Metadata) -> Result<Self> {
    Ok(Self {
      version: value.version,
      authors: value.authors,
      vendors: value.vendors,
      description: value.description,
      documentation: value.documentation,
      licenses: value.licenses,
      icon: value.icon.map(TryInto::try_into).transpose()?,
    })
  }
}
