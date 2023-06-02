use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
mod types;

use flow_expression_parser::ast::{self, InstancePort, InstanceTarget};
use option_utils::OptionUtils;

// use flow_expression_parser::parse_id;
use crate::app_config::{
  AppConfiguration,
  CliConfig,
  HttpRouterConfig,
  HttpTriggerConfig,
  ProxyRouterConfig,
  RawRouterConfig,
  ResourceBinding,
  ResourceDefinition,
  RestRouterConfig,
  StaticRouterConfig,
  TcpPort,
  TimeTriggerConfig,
  TriggerDefinition,
  UdpPort,
};
use crate::component_config::{CompositeComponentImplementation, WasmComponentImplementation};
use crate::config::common::{
  test_case,
  ComponentDefinition,
  ComponentOperationExpression,
  HostConfig,
  OperationSignature,
};
use crate::config::components::{self, ComponentReference, GrpcUrlComponent, ManifestComponent};
use crate::config::package_definition::{PackageConfig, RegistryConfig};
use crate::config::{
  self,
  test_config,
  types_config,
  ComponentConfiguration,
  ComponentImplementation,
  HighLevelComponent,
  ScheduleConfig,
};
use crate::error::ManifestError;
use crate::utils::{opt_str_to_ipv4addr, VecMapInto, VecTryMapInto};
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
      cases: value.cases.map_into(),
      config: value.with.map_into(),
      name: value.name,
      source: None,
    })
  }
}

impl TryFrom<config::TestConfiguration> for v1::TestConfiguration {
  type Error = ManifestError;

  fn try_from(value: config::TestConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      with: value.config.map_into(),
      cases: value.cases.into_iter().map(|v| v.into()).collect(),
    })
  }
}

impl TryFrom<v1::TypesConfiguration> for types_config::TypesConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::TypesConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      metadata: value.metadata.try_map_into()?,
      types: value.types.try_map_into()?,
      operations: value
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
      source: None,
      package: value.package.try_map_into()?,
    })
  }
}

impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      metadata: def.metadata.try_map_into()?,
      host: def.host.try_map_into()?,
      name: def.name,
      tests: def.tests.try_map_into()?,
      component: def.component.try_into()?,
      types: def.types.try_map_into()?,
      requires: def
        .requires
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      import: def
        .import
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      resources: def
        .resources
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      cached_types: Default::default(),
      type_cache: Default::default(),
      package: def.package.try_map_into()?,
    })
  }
}

impl TryFrom<ComponentConfiguration> for v1::ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: ComponentConfiguration) -> Result<Self> {
    Ok(v1::ComponentConfiguration {
      metadata: def.metadata.try_map_into()?,
      host: def.host.try_map_into()?,
      name: def.name,
      requires: def
        .requires
        .into_values()
        .map(|v| v.try_into())
        .collect::<Result<_>>()?,
      import: def.import.into_values().map(|v| v.try_into()).collect::<Result<_>>()?,
      types: def.types.try_map_into()?,
      resources: def.resources.into_values().map(|v| v.into()).collect(),
      tests: def.tests.try_map_into()?,
      component: def.component.try_into()?,
      package: def.package.try_map_into()?,
    })
  }
}

impl TryFrom<v1::PackageDefinition> for PackageConfig {
  type Error = ManifestError;

  fn try_from(value: v1::PackageDefinition) -> std::result::Result<Self, Self::Error> {
    let registry_config = match value.registry {
      Some(registry_def) => Some(RegistryConfig::try_from(registry_def)?),
      None => None,
    };

    Ok(Self {
      files: value.files.try_map_into()?,
      registry: registry_config,
    })
  }
}

impl TryFrom<PackageConfig> for v1::PackageDefinition {
  type Error = ManifestError;

  fn try_from(value: PackageConfig) -> std::result::Result<Self, Self::Error> {
    let registry_def = match value.registry {
      Some(registry_config) => Some(v1::RegistryDefinition::try_from(registry_config)?),
      None => None,
    };

    Ok(v1::PackageDefinition {
      files: value.files.try_map_into()?,
      registry: registry_def,
    })
  }
}

impl TryFrom<super::helpers::Glob> for config::Glob {
  type Error = ManifestError;

  fn try_from(value: super::helpers::Glob) -> std::result::Result<Self, Self::Error> {
    Ok(Self::new(value.0))
  }
}

impl TryFrom<config::Glob> for super::helpers::Glob {
  type Error = ManifestError;

  fn try_from(value: config::Glob) -> std::result::Result<Self, Self::Error> {
    Ok(Self(value.glob))
  }
}

impl TryFrom<v1::RegistryDefinition> for RegistryConfig {
  type Error = ManifestError;

  fn try_from(value: v1::RegistryDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      host: value.host,
      namespace: value.namespace,
    })
  }
}

impl TryFrom<RegistryConfig> for v1::RegistryDefinition {
  type Error = ManifestError;

  fn try_from(value: RegistryConfig) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      host: value.host,
      namespace: value.namespace,
    })
  }
}

impl TryFrom<v1::WasmComponentConfiguration> for WasmComponentImplementation {
  type Error = ManifestError;
  fn try_from(value: v1::WasmComponentConfiguration) -> Result<Self> {
    Ok(Self {
      reference: value.reference.try_into()?,
      config: value.with.try_map_into()?,
      operations: value
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<v1::InterfaceBinding> for config::BoundInterface {
  type Error = ManifestError;

  fn try_from(value: v1::InterfaceBinding) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      id: value.name,
      kind: value.interface.try_into()?,
    })
  }
}

impl TryFrom<v1::InterfaceDefinition> for config::InterfaceDefinition {
  type Error = ManifestError;

  fn try_from(value: v1::InterfaceDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      operations: value
        .operations
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      types: value.types.try_map_into()?,
    })
  }
}

impl TryFrom<v1::CompositeComponentConfiguration> for CompositeComponentImplementation {
  type Error = ManifestError;
  fn try_from(value: v1::CompositeComponentConfiguration) -> Result<Self> {
    Ok(Self {
      config: value.with.try_map_into()?,
      operations: value
        .operations
        .into_iter()
        .map(|op| Ok((op.name.clone(), op.try_into()?)))
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<CompositeComponentImplementation> for v1::CompositeComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: CompositeComponentImplementation) -> Result<Self> {
    Ok(Self {
      with: value.config.try_map_into()?,
      operations: value
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<WasmComponentImplementation> for v1::WasmComponentConfiguration {
  type Error = ManifestError;
  fn try_from(value: WasmComponentImplementation) -> Result<Self> {
    Ok(Self {
      with: value.config.try_map_into()?,
      operations: value
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
      reference: value.reference.try_into()?,
    })
  }
}

impl TryFrom<config::BoundInterface> for v1::InterfaceBinding {
  type Error = ManifestError;

  fn try_from(value: config::BoundInterface) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.id,
      interface: value.kind.try_into()?,
    })
  }
}

impl TryFrom<config::InterfaceDefinition> for v1::InterfaceDefinition {
  type Error = ManifestError;

  fn try_from(value: config::InterfaceDefinition) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      operations: value
        .operations
        .into_values()
        .map(|op| op.try_into())
        .collect::<Result<_>>()?,
      types: value.types.try_map_into()?,
    })
  }
}

impl TryFrom<v1::ComponentOperationExpression> for ComponentOperationExpression {
  type Error = ManifestError;
  fn try_from(literal: v1::ComponentOperationExpression) -> Result<Self> {
    Ok(Self {
      name: literal.name,
      component: literal.component.try_into()?,
      config: literal.with.map_into(),
    })
  }
}

impl TryFrom<v1::AppConfiguration> for AppConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::AppConfiguration) -> Result<Self> {
    Ok(AppConfiguration {
      source: None,
      metadata: def.metadata.try_map_into()?,
      name: def.name,
      import: def
        .import
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      resources: def
        .resources
        .into_iter()
        .map(|v| Ok((v.name.clone(), v.try_into()?)))
        .collect::<Result<_>>()?,
      triggers: def.triggers.into_iter().map(|v| v.try_into()).collect::<Result<_>>()?,
      cached_types: Default::default(),
      type_cache: Default::default(),
      package: def.package.try_map_into()?,
    })
  }
}

impl TryFrom<AppConfiguration> for v1::AppConfiguration {
  type Error = ManifestError;

  fn try_from(value: AppConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      metadata: value.metadata.try_map_into()?,
      name: value.name,
      import: value
        .import
        .into_values()
        .map(|v| v.try_into())
        .collect::<Result<_>>()?,
      resources: value.resources.into_values().map(|v| v.into()).collect(),
      triggers: value.triggers.try_map_into()?,
      package: value.package.try_map_into()?,
    })
  }
}

impl TryFrom<TriggerDefinition> for v1::TriggerDefinition {
  type Error = ManifestError;
  fn try_from(value: TriggerDefinition) -> Result<Self> {
    Ok(match value {
      TriggerDefinition::Http(v) => v1::TriggerDefinition::HttpTrigger(v.try_into()?),
      TriggerDefinition::Cli(v) => v1::TriggerDefinition::CliTrigger(v.try_into()?),
      TriggerDefinition::Time(v) => v1::TriggerDefinition::TimeTrigger(v.try_into()?),
    })
  }
}

impl TryFrom<TimeTriggerConfig> for v1::TimeTrigger {
  type Error = ManifestError;
  fn try_from(value: TimeTriggerConfig) -> Result<Self> {
    let payload: Result<Vec<v1::OperationInput>> = value.payload.try_map_into();

    Ok(Self {
      schedule: value.schedule.try_into()?,
      operation: value.operation.try_into()?,
      payload: payload?,
    })
  }
}

impl TryFrom<ScheduleConfig> for v1::Schedule {
  type Error = ManifestError;
  fn try_from(value: ScheduleConfig) -> Result<Self> {
    Ok(Self {
      cron: value.cron,
      repeat: value.repeat,
    })
  }
}

impl TryFrom<v1::Schedule> for ScheduleConfig {
  type Error = ManifestError;
  fn try_from(value: v1::Schedule) -> Result<Self> {
    Ok(Self {
      cron: value.cron,
      repeat: value.repeat,
    })
  }
}

// Implement conversion from OperationInputConfig to v1::OperationInput
impl TryFrom<config::OperationInputConfig> for v1::OperationInput {
  type Error = ManifestError;

  fn try_from(value: config::OperationInputConfig) -> Result<Self> {
    Ok(v1::OperationInput {
      name: value.name,
      value: value.value,
    })
  }
}

// Implement conversion from v1::OperationInput to OperationInputConfig
impl TryFrom<v1::OperationInput> for config::OperationInputConfig {
  type Error = ManifestError;

  fn try_from(value: v1::OperationInput) -> Result<Self> {
    Ok(config::OperationInputConfig {
      name: value.name,
      value: value.value,
    })
  }
}

impl TryFrom<CliConfig> for v1::CliTrigger {
  type Error = ManifestError;
  fn try_from(value: CliConfig) -> Result<Self> {
    Ok(Self {
      operation: value.operation.try_into()?,
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
      HttpRouterConfig::StaticRouter(v) => v1::HttpRouter::StaticRouter(v.try_into()?),
      HttpRouterConfig::ProxyRouter(v) => v1::HttpRouter::ProxyRouter(v.try_into()?),
    })
  }
}

impl TryFrom<ProxyRouterConfig> for v1::ProxyRouter {
  type Error = ManifestError;
  fn try_from(value: ProxyRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      url: value.url,
      strip_path: value.strip_path,
      middleware: value.middleware.try_map_into()?,
    })
  }
}

impl TryFrom<StaticRouterConfig> for v1::StaticRouter {
  type Error = ManifestError;
  fn try_from(value: StaticRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      volume: value.volume,
      fallback: value.fallback,
      middleware: value.middleware.try_map_into()?,
    })
  }
}

impl TryFrom<RawRouterConfig> for v1::RawRouter {
  type Error = ManifestError;
  fn try_from(value: RawRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      codec: value.codec.map_into(),
      operation: value.operation.try_into()?,
      middleware: value.middleware.try_map_into()?,
    })
  }
}

impl TryFrom<RestRouterConfig> for v1::RestRouter {
  type Error = ManifestError;
  fn try_from(value: RestRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      tools: value.tools.try_map_into()?,
      routes: value.routes.try_map_into()?,
      middleware: value.middleware.try_map_into()?,
      info: value.info.try_map_into()?,
    })
  }
}

impl TryFrom<config::Middleware> for v1::Middleware {
  type Error = ManifestError;

  fn try_from(value: config::Middleware) -> Result<Self> {
    Ok(Self {
      request: value.request.try_map_into()?,
      response: value.response.try_map_into()?,
    })
  }
}

impl TryFrom<v1::Middleware> for config::Middleware {
  type Error = ManifestError;

  fn try_from(value: v1::Middleware) -> Result<Self> {
    Ok(Self {
      request: value.request.try_map_into()?,
      response: value.response.try_map_into()?,
    })
  }
}

impl TryFrom<config::Tools> for v1::Tools {
  type Error = ManifestError;

  fn try_from(value: config::Tools) -> std::result::Result<Self, Self::Error> {
    Ok(Self { openapi: value.openapi })
  }
}

impl TryFrom<v1::Tools> for config::Tools {
  type Error = ManifestError;

  fn try_from(value: v1::Tools) -> std::result::Result<Self, Self::Error> {
    Ok(Self { openapi: value.openapi })
  }
}

impl TryFrom<v1::Route> for config::RestRoute {
  type Error = ManifestError;

  fn try_from(value: v1::Route) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      methods: value.methods,
      uri: value.uri,
      operation: value.operation.try_into()?,
      description: value.description,
      summary: value.summary,
    })
  }
}

impl TryFrom<config::RestRoute> for v1::Route {
  type Error = ManifestError;

  fn try_from(value: config::RestRoute) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      methods: value.methods,
      uri: value.uri,
      operation: value.operation.try_into()?,
      description: value.description,
      summary: value.summary,
    })
  }
}

impl TryFrom<v1::Info> for config::Info {
  type Error = ManifestError;

  fn try_from(value: v1::Info) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      title: value.title,
      description: value.description,
      tos: value.tos,
      contact: value.contact.try_map_into()?,
      license: value.license.try_map_into()?,
      version: value.version,
      documentation: value.documentation.try_map_into()?,
    })
  }
}

impl TryFrom<config::Info> for v1::Info {
  type Error = ManifestError;

  fn try_from(value: config::Info) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      title: value.title,
      description: value.description,
      tos: value.tos,
      contact: value.contact.try_map_into()?,
      license: value.license.try_map_into()?,
      version: value.version,
      documentation: value.documentation.try_map_into()?,
    })
  }
}

impl TryFrom<v1::Documentation> for config::Documentation {
  type Error = ManifestError;

  fn try_from(value: v1::Documentation) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      description: value.description,
      url: value.url,
    })
  }
}

impl TryFrom<config::Documentation> for v1::Documentation {
  type Error = ManifestError;

  fn try_from(value: config::Documentation) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      description: value.description,
      url: value.url,
    })
  }
}

impl TryFrom<v1::Contact> for config::Contact {
  type Error = ManifestError;

  fn try_from(value: v1::Contact) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      url: value.url,
      email: value.email,
    })
  }
}

impl TryFrom<config::Contact> for v1::Contact {
  type Error = ManifestError;

  fn try_from(value: config::Contact) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      url: value.url,
      email: value.email,
    })
  }
}

impl TryFrom<v1::License> for config::License {
  type Error = ManifestError;

  fn try_from(value: v1::License) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      url: value.url,
    })
  }
}

impl TryFrom<config::License> for v1::License {
  type Error = ManifestError;

  fn try_from(value: config::License) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      url: value.url,
    })
  }
}

impl TryFrom<ComponentOperationExpression> for v1::ComponentOperationExpression {
  type Error = ManifestError;
  fn try_from(value: ComponentOperationExpression) -> Result<Self> {
    Ok(Self {
      name: value.name,
      component: value.component.try_into()?,
      with: value.config.map_into(),
    })
  }
}

impl From<ResourceDefinition> for v1::ResourceDefinition {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::TcpPort(v) => v1::ResourceDefinition::TcpPort(v.into()),
      ResourceDefinition::UdpPort(v) => v1::ResourceDefinition::UdpPort(v.into()),
      ResourceDefinition::Url(v) => v1::ResourceDefinition::Url(v.into()),
      ResourceDefinition::Volume(v) => v1::ResourceDefinition::Volume(v.into()),
    }
  }
}

impl From<config::UrlResource> for v1::Url {
  fn from(value: config::UrlResource) -> Self {
    Self { url: value.to_string() }
  }
}

impl From<config::Volume> for v1::Volume {
  fn from(value: config::Volume) -> Self {
    Self {
      path: value.path.to_string(),
    }
  }
}

impl From<UdpPort> for v1::UdpPort {
  fn from(value: UdpPort) -> Self {
    Self {
      port: value.port,
      address: value.host,
    }
  }
}

impl From<TcpPort> for v1::TcpPort {
  fn from(value: TcpPort) -> Self {
    Self {
      port: value.port,
      address: value.host,
    }
  }
}

impl TryFrom<crate::v1::CompositeOperationDefinition> for config::FlowOperation {
  type Error = ManifestError;

  fn try_from(op: crate::v1::CompositeOperationDefinition) -> Result<Self> {
    let instances: Result<HashMap<String, config::InstanceReference>> = op
      .uses
      .into_iter()
      .map(|v| Ok((v.name.clone(), v.try_into()?)))
      .collect();
    let expressions: Result<Vec<ast::FlowExpression>> = op.flow.into_iter().map(TryInto::try_into).collect();
    Ok(Self {
      name: op.name,
      inputs: op.inputs.try_map_into()?,
      outputs: op.outputs.try_map_into()?,
      instances: instances?,
      expressions: expressions?,
      config: op.with.try_map_into()?,
      flows: op.operations.try_map_into()?,
    })
  }
}

impl TryFrom<v1::FlowExpression> for ast::FlowExpression {
  type Error = ManifestError;

  fn try_from(expr: v1::FlowExpression) -> Result<Self> {
    Ok(match expr {
      v1::FlowExpression::ConnectionDefinition(v) => ast::FlowExpression::connection(v.try_into()?),
      v1::FlowExpression::BlockExpression(v) => ast::FlowExpression::block(v.try_into()?),
    })
  }
}

impl TryFrom<v1::ConnectionDefinition> for ast::ConnectionExpression {
  type Error = ManifestError;

  fn try_from(expr: v1::ConnectionDefinition) -> Result<Self> {
    Ok(Self::new(expr.from.try_into()?, expr.to.try_into()?))
  }
}

impl TryFrom<v1::BlockExpression> for ast::BlockExpression {
  type Error = ManifestError;

  fn try_from(value: v1::BlockExpression) -> std::result::Result<Self, Self::Error> {
    Ok(Self::new(value.expressions.try_map_into()?))
  }
}

impl TryFrom<v1::ComponentKind> for ComponentImplementation {
  type Error = ManifestError;
  fn try_from(value: v1::ComponentKind) -> Result<Self> {
    Ok(match value {
      v1::ComponentKind::CompositeComponentConfiguration(v) => ComponentImplementation::Composite(v.try_into()?),
      v1::ComponentKind::WasmComponentConfiguration(v) => ComponentImplementation::Wasm(v.try_into()?),
      v1::ComponentKind::HttpClientComponent(v) => ComponentImplementation::HttpClient(v.try_into()?),
      v1::ComponentKind::SqlComponent(v) => ComponentImplementation::Sql(v.try_into()?),
    })
  }
}

impl TryFrom<ComponentImplementation> for v1::ComponentKind {
  type Error = ManifestError;
  fn try_from(value: ComponentImplementation) -> Result<Self> {
    Ok(match value {
      ComponentImplementation::Composite(v) => v1::ComponentKind::CompositeComponentConfiguration(v.try_into()?),
      ComponentImplementation::Wasm(v) => v1::ComponentKind::WasmComponentConfiguration(v.try_into()?),
      ComponentImplementation::Sql(v) => v1::ComponentKind::SqlComponent(v.try_into()?),
      ComponentImplementation::HttpClient(v) => v1::ComponentKind::HttpClientComponent(v.try_into()?),
    })
  }
}

impl TryFrom<crate::v1::OperationDefinition> for OperationSignature {
  type Error = ManifestError;

  fn try_from(op: crate::v1::OperationDefinition) -> Result<Self> {
    Ok(Self {
      name: op.name,
      config: op.with.try_map_into()?,
      inputs: op.inputs.try_map_into()?,
      outputs: op.outputs.try_map_into()?,
    })
  }
}

impl TryFrom<OperationSignature> for crate::v1::OperationDefinition {
  type Error = ManifestError;

  fn try_from(op: OperationSignature) -> Result<Self> {
    Ok(Self {
      name: op.name,
      with: op.config.try_map_into()?,
      inputs: op.inputs.try_map_into()?,
      outputs: op.outputs.try_map_into()?,
    })
  }
}

impl TryFrom<config::ImportBinding> for v1::ImportBinding {
  type Error = ManifestError;
  fn try_from(def: config::ImportBinding) -> Result<Self> {
    Ok(Self {
      name: def.id,
      component: def.kind.try_into()?,
    })
  }
}

impl TryFrom<config::ImportDefinition> for v1::ImportDefinition {
  type Error = ManifestError;
  fn try_from(def: config::ImportDefinition) -> Result<Self> {
    Ok(match def {
      crate::common::ImportDefinition::Component(comp) => match comp {
        ComponentDefinition::Native(_) => unreachable!("Native components are not allowed in imports"),
        #[allow(deprecated)]
        ComponentDefinition::Wasm(_) => unreachable!("Wasm components are not allowed in v1 imports"),
        ComponentDefinition::Reference(_) => unreachable!("Component references can't exist in v1 imports"),
        ComponentDefinition::GrpcUrl(_) => unreachable!("GrpcUrl components are not allowed in v1 imports"),
        ComponentDefinition::Manifest(c) => v1::ImportDefinition::ManifestComponent(c.try_into()?),
        ComponentDefinition::HighLevelComponent(c) => match c {
          HighLevelComponent::Sql(c) => v1::ImportDefinition::SqlComponent(c.try_into()?),
          HighLevelComponent::HttpClient(c) => v1::ImportDefinition::HttpClientComponent(c.try_into()?),
        },
      },
      crate::common::ImportDefinition::Types(c) => v1::ImportDefinition::TypesComponent(c.try_into()?),
    })
  }
}

impl From<ResourceBinding> for v1::ResourceBinding {
  fn from(value: ResourceBinding) -> Self {
    Self {
      name: value.id,
      resource: value.kind.into(),
    }
  }
}

impl TryFrom<config::components::TypesComponent> for v1::TypesComponent {
  type Error = ManifestError;
  fn try_from(value: config::components::TypesComponent) -> Result<Self> {
    Ok(Self {
      reference: value.reference.try_into()?,
      types: value.types,
    })
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
      ComponentDefinition::HighLevelComponent(v) => match v {
        config::HighLevelComponent::Sql(v) => Self::SqlComponent(v.try_into()?),
        config::HighLevelComponent::HttpClient(v) => Self::HttpClientComponent(v.try_into()?),
      },
    };
    Ok(def)
  }
}

impl TryFrom<ManifestComponent> for v1::ManifestComponent {
  type Error = ManifestError;
  fn try_from(def: ManifestComponent) -> Result<Self> {
    Ok(Self {
      reference: def.reference.try_into()?,
      with: def.config.map_into(),
      provide: def.provide,
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
      with: def.config.map_into(),
    }
  }
}

impl TryFrom<config::components::HttpClientComponentConfig> for v1::HttpClientComponent {
  type Error = ManifestError;
  fn try_from(value: config::components::HttpClientComponentConfig) -> Result<Self> {
    Ok(Self {
      resource: value.resource,
      codec: value.codec.map_into(),
      operations: value.operations.try_map_into()?,
    })
  }
}

impl From<config::components::Codec> for v1::Codec {
  fn from(value: config::components::Codec) -> Self {
    match value {
      config::components::Codec::Json => Self::Json,
      config::components::Codec::Raw => Self::Raw,
      config::components::Codec::FormData => Self::FormData,
    }
  }
}

impl From<v1::Codec> for config::components::Codec {
  fn from(value: v1::Codec) -> Self {
    match value {
      v1::Codec::Json => Self::Json,
      v1::Codec::Raw => Self::Raw,
      v1::Codec::FormData => Self::FormData,
    }
  }
}

impl TryFrom<config::FlowOperation> for v1::CompositeOperationDefinition {
  type Error = ManifestError;

  fn try_from(value: config::FlowOperation) -> std::result::Result<Self, Self::Error> {
    let instances: Vec<v1::OperationInstance> = value.instances.into_iter().map(from_wat).collect();
    let connections: Result<Vec<v1::FlowExpression>> = value.expressions.try_map_into();
    Ok(Self {
      name: value.name,
      inputs: value.inputs.try_map_into()?,
      outputs: value.outputs.try_map_into()?,
      with: value.config.try_map_into()?,
      uses: instances,
      flow: connections?,
      operations: value.flows.try_map_into()?,
    })
  }
}

impl TryFrom<ast::FlowExpression> for v1::FlowExpression {
  type Error = ManifestError;

  fn try_from(value: ast::FlowExpression) -> std::result::Result<Self, Self::Error> {
    match value {
      ast::FlowExpression::ConnectionExpression(c) => Ok(Self::ConnectionDefinition((*c).try_into()?)),
      ast::FlowExpression::BlockExpression(c) => Ok(Self::BlockExpression(c.try_into()?)),
    }
  }
}

impl TryFrom<ast::BlockExpression> for v1::BlockExpression {
  type Error = ManifestError;

  fn try_from(value: ast::BlockExpression) -> std::result::Result<Self, Self::Error> {
    let expressions = value.into_parts();
    Ok(Self {
      expressions: expressions.try_map_into()?,
    })
  }
}

impl TryFrom<ast::ConnectionExpression> for v1::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(value: ast::ConnectionExpression) -> std::result::Result<Self, Self::Error> {
    let (from, to) = value.into_parts();
    Ok(Self {
      from: from.try_into()?,
      to: to.try_into()?,
    })
  }
}

impl TryFrom<ast::ConnectionTargetExpression> for v1::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(value: ast::ConnectionTargetExpression) -> std::result::Result<Self, Self::Error> {
    let (instance, port, data) = value.into_parts();
    Ok(Self {
      data,
      instance: instance.to_string(),
      port: port.to_string(),
    })
  }
}

fn from_wat(value: (String, config::InstanceReference)) -> v1::OperationInstance {
  let id = value.0;
  let value = value.1;
  v1::OperationInstance {
    name: id,
    operation: v1::ComponentOperationExpression {
      name: value.name,
      component: v1::ComponentDefinition::ComponentReference(v1::ComponentReference { id: value.component_id }),
      with: value.data.map_into(),
    },
    with: None,
  }
}

impl TryFrom<crate::v1::ComponentDefinition> for ComponentDefinition {
  type Error = ManifestError;
  fn try_from(def: crate::v1::ComponentDefinition) -> Result<Self> {
    let res = match def {
      v1::ComponentDefinition::GrpcUrlComponent(v) => ComponentDefinition::GrpcUrl(GrpcUrlComponent {
        url: v.url,
        config: v.with.map_into(),
      }),
      v1::ComponentDefinition::ManifestComponent(v) => ComponentDefinition::Manifest(ManifestComponent {
        reference: v.reference.try_into()?,
        config: v.with.map_into(),
        provide: v.provide,
      }),
      v1::ComponentDefinition::ComponentReference(v) => ComponentDefinition::Reference(ComponentReference { id: v.id }),
      v1::ComponentDefinition::SqlComponent(v) => {
        ComponentDefinition::HighLevelComponent(HighLevelComponent::Sql(v.try_into()?))
      }
      v1::ComponentDefinition::HttpClientComponent(v) => {
        ComponentDefinition::HighLevelComponent(HighLevelComponent::HttpClient(v.try_into()?))
      }
    };
    Ok(res)
  }
}

impl TryFrom<crate::v1::OperationInstance> for config::InstanceReference {
  type Error = ManifestError;
  fn try_from(def: crate::v1::OperationInstance) -> Result<Self> {
    let ns = def.operation.component.component_id().unwrap_or("<anonymous>");
    let name = def.operation.name;
    Ok(config::InstanceReference {
      component_id: ns.to_owned(),
      name,
      data: def.with.map_into(),
    })
  }
}

impl TryFrom<&crate::v1::ConnectionDefinition> for ast::ConnectionExpression {
  type Error = ManifestError;

  fn try_from(def: &crate::v1::ConnectionDefinition) -> Result<Self> {
    let from: ast::ConnectionTargetExpression = def.from.clone().try_into()?;
    let to: ast::ConnectionTargetExpression = def.to.clone().try_into()?;
    Ok(ast::ConnectionExpression::new(from, to))
  }
}

impl TryFrom<crate::v1::HostConfig> for HostConfig {
  type Error = ManifestError;
  fn try_from(def: crate::v1::HostConfig) -> Result<Self> {
    Ok(Self {
      allow_latest: def.allow_latest,
      insecure_registries: def.insecure_registries,
      timeout: Duration::from_millis(def.timeout),
      rpc: def.rpc.try_map_into()?,
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
      rpc: def.rpc.try_map_into()?,
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
      pem: def.pem.try_map_into()?,
      key: def.key.try_map_into()?,
      ca: def.ca.try_map_into()?,
    })
  }
}

impl TryFrom<crate::v1::ConnectionTargetDefinition> for ast::ConnectionTargetExpression {
  type Error = ManifestError;

  fn try_from(def: crate::v1::ConnectionTargetDefinition) -> Result<Self> {
    Ok(ast::ConnectionTargetExpression::new_default(
      InstanceTarget::from_str(&def.instance)?,
      InstancePort::from_str(&def.port)?,
      def.data,
    ))
  }
}

impl TryFrom<v1::ResourceDefinition> for ResourceDefinition {
  type Error = ManifestError;
  fn try_from(value: v1::ResourceDefinition) -> Result<Self> {
    Ok(match value {
      v1::ResourceDefinition::TcpPort(v) => Self::TcpPort(v.into()),
      v1::ResourceDefinition::UdpPort(v) => Self::UdpPort(v.into()),
      v1::ResourceDefinition::Url(v) => Self::Url(v.url.try_into()?),
      v1::ResourceDefinition::Volume(v) => Self::Volume(v.into()),
    })
  }
}

impl From<v1::Volume> for config::Volume {
  fn from(value: v1::Volume) -> Self {
    config::Volume::new(value.path)
  }
}

impl From<v1::TcpPort> for TcpPort {
  fn from(value: v1::TcpPort) -> Self {
    Self {
      port: value.port,
      host: value.address,
    }
  }
}

impl From<v1::UdpPort> for UdpPort {
  fn from(value: v1::UdpPort) -> Self {
    Self {
      port: value.port,
      host: value.address,
    }
  }
}

impl TryFrom<v1::TriggerDefinition> for TriggerDefinition {
  type Error = ManifestError;
  fn try_from(trigger: v1::TriggerDefinition) -> Result<Self> {
    let rv = match trigger {
      v1::TriggerDefinition::CliTrigger(cli) => Self::Cli(CliConfig {
        operation: cli.operation.try_into()?,
      }),
      v1::TriggerDefinition::HttpTrigger(v) => Self::Http(HttpTriggerConfig {
        resource: v.resource,
        routers: v.routers.try_map_into()?,
      }),
      v1::TriggerDefinition::TimeTrigger(time) => Self::Time(TimeTriggerConfig {
        schedule: time.schedule.try_into()?,
        operation: time.operation.try_into()?,
        payload: time.payload.try_map_into()?,
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
        codec: v.codec.map_into(),
        operation: v.operation.try_into()?,
        middleware: v.middleware.try_map_into()?,
      }),
      v1::HttpRouter::RestRouter(v) => Self::RestRouter(RestRouterConfig {
        path: v.path,
        tools: v.tools.try_map_into()?,
        routes: v.routes.try_map_into()?,
        info: v.info.try_map_into()?,
        middleware: v.middleware.try_map_into()?,
      }),
      v1::HttpRouter::StaticRouter(v) => Self::StaticRouter(StaticRouterConfig {
        path: v.path,
        volume: v.volume,
        fallback: v.fallback,
        middleware: v.middleware.try_map_into()?,
      }),
      v1::HttpRouter::ProxyRouter(v) => Self::ProxyRouter(ProxyRouterConfig {
        path: v.path,
        url: v.url,
        strip_path: v.strip_path,
        middleware: v.middleware.try_map_into()?,
      }),
    };
    Ok(rv)
  }
}

impl TryFrom<v1::ImportBinding> for config::ImportBinding {
  type Error = ManifestError;
  fn try_from(value: v1::ImportBinding) -> Result<Self> {
    Ok(Self {
      id: value.name,
      kind: value.component.try_into()?,
    })
  }
}

impl TryFrom<v1::ImportDefinition> for config::ImportDefinition {
  type Error = ManifestError;
  fn try_from(value: v1::ImportDefinition) -> Result<Self> {
    Ok(match value {
      v1::ImportDefinition::TypesComponent(c) => config::ImportDefinition::Types(c.try_into()?),
      v1::ImportDefinition::ManifestComponent(c) => {
        let c = v1::ComponentDefinition::ManifestComponent(c);
        config::ImportDefinition::Component(c.try_into()?)
      }
      v1::ImportDefinition::SqlComponent(c) => config::ImportDefinition::Component(
        config::ComponentDefinition::HighLevelComponent(config::HighLevelComponent::Sql(c.try_into()?)),
      ),
      v1::ImportDefinition::HttpClientComponent(c) => config::ImportDefinition::Component(
        config::ComponentDefinition::HighLevelComponent(config::HighLevelComponent::HttpClient(c.try_into()?)),
      ),
    })
  }
}

impl TryFrom<v1::TypesComponent> for config::components::TypesComponent {
  type Error = ManifestError;

  fn try_from(value: v1::TypesComponent) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      reference: value.reference.try_into()?,
      types: value.types,
    })
  }
}

impl TryFrom<v1::ResourceBinding> for ResourceBinding {
  type Error = ManifestError;
  fn try_from(value: v1::ResourceBinding) -> Result<Self> {
    Ok(Self {
      id: value.name,
      kind: value.resource.try_into()?,
    })
  }
}

impl From<v1::TestDefinition> for test_case::TestCase {
  fn from(value: v1::TestDefinition) -> Self {
    Self {
      name: value.name,
      operation: value.operation,
      inputs: value.input.map_into(),
      outputs: value.output.map_into(),
      inherent: value.inherent.map_into(),
      config: value.with.map_into(),
    }
  }
}

impl From<v1::PacketData> for test_case::TestPacket {
  fn from(value: v1::PacketData) -> Self {
    match value {
      v1::PacketData::SuccessPacket(v) => test_case::TestPacket::SuccessPacket(v.into()),
      v1::PacketData::ErrorPacket(v) => test_case::TestPacket::ErrorPacket(v.into()),
    }
  }
}

impl From<v1::SuccessPacket> for config::SuccessPayload {
  fn from(value: v1::SuccessPacket) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map_into(),
      data: value.value,
    }
  }
}

impl From<v1::ErrorPacket> for config::ErrorPayload {
  fn from(value: v1::ErrorPacket) -> Self {
    Self {
      port: value.name,
      flags: value.flags.map_into(),
      error: value.error,
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
      input: value.inputs.map_into(),
      output: value.outputs.map_into(),
      inherent: value.inherent.map_into(),
      with: value.config.map_into(),
    }
  }
}

impl From<test_case::ErrorPayload> for v1::ErrorPacket {
  fn from(value: test_case::ErrorPayload) -> Self {
    Self {
      name: value.port,
      flags: value.flags.map_into(),
      error: value.error,
    }
  }
}

impl From<test_case::SuccessPayload> for v1::SuccessPacket {
  fn from(value: test_case::SuccessPayload) -> Self {
    Self {
      name: value.port,
      flags: value.flags.map_into(),
      value: value.data,
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
      test_case::TestPacket::SuccessPacket(v) => v1::PacketData::SuccessPacket(v.into()),
      test_case::TestPacket::ErrorPacket(v) => v1::PacketData::ErrorPacket(v.into()),
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
      icon: value.icon.try_map_into()?,
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
      icon: value.icon.try_map_into()?,
    })
  }
}

impl TryFrom<v1::SqlComponent> for components::SqlComponentConfig {
  type Error = crate::Error;
  fn try_from(value: v1::SqlComponent) -> Result<Self> {
    Ok(Self {
      resource: value.resource,
      tls: value.tls,
      operations: value
        .operations
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<v1::SqlOperationDefinition> for components::SqlOperationDefinition {
  type Error = crate::Error;
  fn try_from(value: v1::SqlOperationDefinition) -> Result<Self> {
    Ok(Self {
      name: value.name,
      inputs: value.inputs.try_map_into()?,
      outputs: value.outputs.try_map_into()?,
      query: value.query,
      arguments: value.arguments,
      config: value.with.try_map_into()?,
    })
  }
}

impl TryFrom<v1::HttpClientOperationDefinition> for components::HttpClientOperationDefinition {
  type Error = crate::Error;
  fn try_from(value: v1::HttpClientOperationDefinition) -> Result<Self> {
    Ok(Self {
      name: value.name,
      codec: value.codec.map_into(),
      inputs: value.inputs.try_map_into()?,
      path: value.path,
      body: value.body,
      method: value.method.into(),
      config: value.with.try_map_into()?,
      headers: value.headers,
    })
  }
}

impl From<v1::HttpMethod> for components::HttpMethod {
  fn from(value: v1::HttpMethod) -> Self {
    match value {
      v1::HttpMethod::Get => Self::Get,
      v1::HttpMethod::Post => Self::Post,
      v1::HttpMethod::Put => Self::Put,
      v1::HttpMethod::Delete => Self::Delete,
    }
  }
}

impl From<components::HttpMethod> for v1::HttpMethod {
  fn from(value: components::HttpMethod) -> Self {
    match value {
      components::HttpMethod::Get => Self::Get,
      components::HttpMethod::Post => Self::Post,
      components::HttpMethod::Put => Self::Put,
      components::HttpMethod::Delete => Self::Delete,
    }
  }
}

impl TryFrom<v1::HttpClientComponent> for components::HttpClientComponentConfig {
  type Error = crate::Error;
  fn try_from(value: v1::HttpClientComponent) -> Result<Self> {
    Ok(Self {
      resource: value.resource,
      codec: value.codec.map_into(),
      operations: value.operations.try_map_into()?,
    })
  }
}

impl TryFrom<components::SqlComponentConfig> for v1::SqlComponent {
  type Error = crate::Error;
  fn try_from(value: components::SqlComponentConfig) -> Result<Self> {
    Ok(Self {
      resource: value.resource,
      tls: value.tls,
      operations: value.operations.try_map_into()?,
    })
  }
}

impl TryFrom<components::SqlOperationDefinition> for v1::SqlOperationDefinition {
  type Error = crate::Error;
  fn try_from(value: components::SqlOperationDefinition) -> Result<Self> {
    Ok(Self {
      name: value.name,
      inputs: value.inputs.try_map_into()?,
      outputs: value.outputs.try_map_into()?,
      query: value.query,
      arguments: value.arguments,
      with: value.config.try_map_into()?,
    })
  }
}

impl TryFrom<components::HttpClientOperationDefinition> for v1::HttpClientOperationDefinition {
  type Error = crate::Error;
  fn try_from(value: components::HttpClientOperationDefinition) -> Result<Self> {
    Ok(Self {
      name: value.name,
      inputs: value.inputs.try_map_into()?,
      path: value.path,
      body: value.body,
      codec: value.codec.map_into(),
      method: value.method.into(),
      with: value.config.try_map_into()?,
      headers: value.headers,
    })
  }
}
