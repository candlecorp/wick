use std::collections::HashMap;
use std::time::Duration;

// use flow_expression_parser::parse_id;
use crate::app_config::BoundResource;
use crate::component_definition::{ComponentImplementation, ComponentOperationExpression};
use crate::error::ManifestError;
use crate::flow_definition::{PortReference, SenderData};
use crate::host_definition::HostConfig;
use crate::utils::{opt_str_to_ipv4addr, opt_str_to_pathbuf};
use crate::{
  test_case,
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

impl TryFrom<v1::V1ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::V1ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      format: def.format,
      main: def.main.map(|v| v.into()),
      types: def.types,
      version: def.metadata.unwrap_or(v1::ComponentMetadata::default()).version,
      host: def.host.try_into()?,
      name: def.name,
      import: def.import.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      labels: def.labels,
      tests: def.tests.into_iter().map(|v| v.into()).collect(),
      operations: def
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<ComponentConfiguration> for v1::V1ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: ComponentConfiguration) -> Result<Self> {
    Ok(v1::V1ComponentConfiguration {
      format: def.format,
      types: def.types,
      main: def.main.map(|v| v.into()),
      metadata: Some(v1::ComponentMetadata { version: def.version }),
      host: def.host.try_into()?,
      name: def.name,
      import: def.import.into_values().map(|v| v.into()).collect(),
      labels: def.labels,
      tests: def.tests.into_iter().map(|v| v.into()).collect(),
      operations: def
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
    })
  }
}

impl From<v1::ComponentOperationExpression> for ComponentOperationExpression {
  fn from(literal: v1::ComponentOperationExpression) -> Self {
    Self {
      operation: literal.name,
      component: literal.component.into(),
    }
  }
}

impl TryFrom<v1::V1AppConfiguration> for AppConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::V1AppConfiguration) -> Result<Self> {
    Ok(AppConfiguration {
      source: None,
      format: def.format,
      version: def.metadata.unwrap_or_default().version,
      name: def.name,
      host: def.host.try_into()?,
      import: def.import.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      resources: def.resources.into_iter().map(|v| (v.name.clone(), v.into())).collect(),
      triggers: def.triggers.into_iter().map(|v| v.into()).collect(),
    })
  }
}

impl TryFrom<AppConfiguration> for v1::V1AppConfiguration {
  type Error = ManifestError;

  fn try_from(value: AppConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      format: value.format,
      metadata: Some(v1::AppMetadata { version: value.version }),
      name: value.name,
      import: value.import.into_values().map(|v| v.into()).collect(),
      resources: value.resources.into_values().map(|v| v.into()).collect(),
      triggers: value.triggers.into_iter().map(|v| v.into()).collect(),
      host: value.host.try_into()?,
    })
  }
}

impl From<TriggerDefinition> for v1::TriggerDefinition {
  fn from(value: TriggerDefinition) -> Self {
    match value {
      TriggerDefinition::Http(v) => v1::TriggerDefinition::HttpTrigger(v.into()),
      TriggerDefinition::Cli(v) => v1::TriggerDefinition::CliTrigger(v.into()),
    }
  }
}

impl From<CliConfig> for v1::CliTrigger {
  fn from(value: CliConfig) -> Self {
    Self {
      operation: value.operation.into(),
      app: value.app.map(|v| v.into()),
    }
  }
}

impl From<HttpTriggerConfig> for v1::HttpTrigger {
  fn from(value: HttpTriggerConfig) -> Self {
    Self {
      resource: value.resource,
      routers: value.routers.into_iter().map(|v| v.into()).collect(),
    }
  }
}

impl From<HttpRouterConfig> for v1::HttpRouter {
  fn from(value: HttpRouterConfig) -> Self {
    match value {
      HttpRouterConfig::RawRouter(v) => v1::HttpRouter::RawRouter(v.into()),
      HttpRouterConfig::RestRouter(v) => v1::HttpRouter::RestRouter(v.into()),
    }
  }
}

impl From<RawRouterConfig> for v1::RawRouter {
  fn from(value: RawRouterConfig) -> Self {
    Self {
      path: value.path,
      operation: value.operation.into(),
    }
  }
}

impl From<RestRouterConfig> for v1::RestRouter {
  fn from(value: RestRouterConfig) -> Self {
    Self {
      path: value.path,
      component: value.component.into(),
    }
  }
}

impl From<ComponentOperationExpression> for v1::ComponentOperationExpression {
  fn from(value: ComponentOperationExpression) -> Self {
    Self {
      name: value.operation,
      component: value.component.into(),
    }
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
impl TryFrom<crate::v1::OperationDefinition> for FlowOperation {
  type Error = ManifestError;

  fn try_from(op: crate::v1::OperationDefinition) -> Result<Self> {
    let instances: Result<HashMap<String, InstanceReference>> = op
      .instances
      .into_iter()
      .map(|v| Ok((v.name.clone(), v.try_into()?)))
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

impl From<BoundComponent> for v1::ComponentBinding {
  fn from(def: BoundComponent) -> Self {
    Self {
      name: def.id,
      component: def.kind.into(),
    }
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

impl From<ComponentDefinition> for v1::ComponentDefinition {
  fn from(kind: ComponentDefinition) -> Self {
    match kind {
      ComponentDefinition::Wasm(wasm) => Self::WasmRsComponent(wasm.into()),
      ComponentDefinition::GrpcUrl(grpc) => Self::GrpcUrlComponent(grpc.into()),
      ComponentDefinition::Native(_) => todo!(),
      ComponentDefinition::Reference(v) => Self::ComponentReference(v.into()),
      ComponentDefinition::Manifest(v) => Self::ManifestComponent(v.into()),
    }
  }
}

impl From<ManifestComponent> for v1::ManifestComponent {
  fn from(def: ManifestComponent) -> Self {
    Self {
      reference: def.reference,
      config: def.config,
    }
  }
}
impl From<WasmComponent> for v1::WasmRsComponent {
  fn from(def: WasmComponent) -> Self {
    Self {
      reference: def.reference,
      config: def.config,
      permissions: def.permissions.into(),
    }
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

impl From<Permissions> for v1::Permissions {
  fn from(value: Permissions) -> Self {
    Self { dirs: value.dirs }
  }
}

impl TryFrom<FlowOperation> for v1::OperationDefinition {
  type Error = ManifestError;

  fn try_from(value: FlowOperation) -> std::result::Result<Self, Self::Error> {
    let instances: Result<Vec<v1::InstanceBinding>> = value
      .instances
      .into_iter()
      .map(|(id, val)| (id, val).try_into())
      .collect();
    let connections: Result<Vec<v1::ConnectionDefinition>> =
      value.connections.into_iter().map(|def| def.try_into()).collect();
    Ok(Self {
      name: value.name,
      inputs: value.inputs,
      outputs: value.outputs,
      instances: instances?,
      flow: connections?,
      components: value.collections,
    })
  }
}

impl TryFrom<ConnectionDefinition> for v1::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(value: ConnectionDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      from: value.from.try_into()?,
      to: value.to.try_into()?,
    })
  }
}

impl TryFrom<ConnectionTargetDefinition> for v1::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(value: ConnectionTargetDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      data: value.data.map(|v| v.inner),
      instance: value.target.instance,
      port: value.target.port,
    })
  }
}

impl TryFrom<(String, InstanceReference)> for v1::InstanceBinding {
  type Error = ManifestError;

  fn try_from(value: (String, InstanceReference)) -> std::result::Result<Self, Self::Error> {
    let id = value.0;
    let value = value.1;
    Ok(Self {
      name: id,
      operation: v1::ComponentOperationExpression {
        name: value.name,
        component: v1::ComponentDefinition::ComponentReference(v1::ComponentReference { id: value.component_id }),
      },
      config: value.data,
    })
  }
}

impl From<crate::v1::ComponentDefinition> for ComponentDefinition {
  fn from(def: crate::v1::ComponentDefinition) -> Self {
    match def {
      crate::v1::ComponentDefinition::WasmRsComponent(v) => ComponentDefinition::Wasm(v.into()),
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

impl TryFrom<crate::v1::InstanceBinding> for InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v1::InstanceBinding) -> Result<Self> {
    let ns = def.operation.component.component_id().unwrap_or("<anonymous>");
    let name = def.operation.name;
    Ok(InstanceReference {
      component_id: ns.to_owned(),
      name,
      data: def.config,
    })
  }
}

impl TryFrom<&crate::v1::ConnectionDefinition> for ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(def: &crate::v1::ConnectionDefinition) -> Result<Self> {
    let from: ConnectionTargetDefinition = def.from.clone().try_into()?;
    let to: ConnectionTargetDefinition = def.to.clone().try_into()?;
    Ok(ConnectionDefinition { from, to })
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

impl TryFrom<HttpConfig> for crate::v1::HttpConfig {
  type Error = ManifestError;
  fn try_from(def: HttpConfig) -> Result<Self> {
    Ok(Self {
      enabled: def.enabled,
      port: def.port,
      address: def.address.map(|v| v.to_string()),
      pem: def.pem.map(|v| v.to_string_lossy().to_string()),
      key: def.key.map(|v| v.to_string_lossy().to_string()),
      ca: def.ca.map(|v| v.to_string_lossy().to_string()),
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

impl From<v1::PayloadData> for crate::test_case::SuccessPayload {
  fn from(value: v1::PayloadData) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map(|v| v.into()),
      data: value.data,
    }
  }
}

impl From<v1::ErrorData> for crate::test_case::ErrorPayload {
  fn from(value: v1::ErrorData) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map(|v| v.into()),
      message: value.message,
    }
  }
}

impl From<v1::PacketFlags> for crate::test_case::PacketFlags {
  fn from(value: v1::PacketFlags) -> Self {
    Self {
      done: value.done,
      open: value.open,
      close: value.close,
    }
  }
}

impl From<v1::InherentData> for crate::test_case::InherentConfig {
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

impl From<v1::LocalDefinition> for ComponentImplementation {
  fn from(value: v1::LocalDefinition) -> Self {
    match value {
      v1::LocalDefinition::WasmRsComponent(v) => Self::Wasm(v.into()),
    }
  }
}

impl From<v1::WasmRsComponent> for WasmComponent {
  fn from(value: v1::WasmRsComponent) -> Self {
    Self {
      reference: value.reference,
      config: value.config,
      permissions: value.permissions.into(),
    }
  }
}
impl From<ComponentImplementation> for v1::LocalDefinition {
  fn from(value: ComponentImplementation) -> Self {
    match value {
      ComponentImplementation::Wasm(v) => Self::WasmRsComponent(v.into()),
    }
  }
}
