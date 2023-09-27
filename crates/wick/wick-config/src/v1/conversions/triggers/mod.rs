use option_utils::OptionUtils;

use crate::error::ManifestError;
use crate::utils::{VecMapInto, VecTryMapInto};
use crate::{config, v1};

type Result<T> = std::result::Result<T, ManifestError>;

impl TryFrom<config::TriggerDefinition> for v1::TriggerDefinition {
  type Error = ManifestError;
  fn try_from(value: config::TriggerDefinition) -> Result<Self> {
    Ok(match value {
      config::TriggerDefinition::Http(v) => v1::TriggerDefinition::HttpTrigger(v.try_into()?),
      config::TriggerDefinition::Cli(v) => v1::TriggerDefinition::CliTrigger(v.try_into()?),
      config::TriggerDefinition::Time(v) => v1::TriggerDefinition::TimeTrigger(v.try_into()?),
      config::TriggerDefinition::WasmCommand(v) => v1::TriggerDefinition::WasmCommandTrigger(v.try_into()?),
    })
  }
}

impl TryFrom<config::WasmCommandConfig> for v1::WasmCommandTrigger {
  type Error = ManifestError;
  fn try_from(value: config::WasmCommandConfig) -> Result<Self> {
    Ok(Self {
      reference: value.reference.try_into()?,
      volumes: value.volumes.try_map_into()?,
    })
  }
}

impl TryFrom<config::TimeTriggerConfig> for v1::TimeTrigger {
  type Error = ManifestError;
  fn try_from(value: config::TimeTriggerConfig) -> Result<Self> {
    let payload: Result<Vec<v1::OperationInput>> = value.payload.try_map_into();

    Ok(Self {
      schedule: value.schedule.try_into()?,
      operation: value.operation.try_into()?,
      payload: payload?,
    })
  }
}

impl TryFrom<config::ScheduleConfig> for v1::Schedule {
  type Error = ManifestError;
  fn try_from(value: config::ScheduleConfig) -> Result<Self> {
    Ok(Self {
      cron: value.cron,
      repeat: value.repeat,
    })
  }
}

impl TryFrom<v1::Schedule> for config::ScheduleConfig {
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

impl TryFrom<config::CliConfig> for v1::CliTrigger {
  type Error = ManifestError;
  fn try_from(value: config::CliConfig) -> Result<Self> {
    Ok(Self {
      operation: value.operation.try_into()?,
    })
  }
}

impl TryFrom<config::HttpTriggerConfig> for v1::HttpTrigger {
  type Error = ManifestError;
  fn try_from(value: config::HttpTriggerConfig) -> Result<Self> {
    Ok(Self {
      resource: value.resource.id().to_owned(),
      routers: value.routers.into_iter().map(|v| v.try_into()).collect::<Result<_>>()?,
    })
  }
}

impl TryFrom<config::HttpRouterConfig> for v1::HttpRouter {
  type Error = ManifestError;
  fn try_from(value: config::HttpRouterConfig) -> Result<Self> {
    Ok(match value {
      config::HttpRouterConfig::RawRouter(v) => v1::HttpRouter::RawRouter(v.try_into()?),
      config::HttpRouterConfig::RestRouter(v) => v1::HttpRouter::RestRouter(v.try_into()?),
      config::HttpRouterConfig::StaticRouter(v) => v1::HttpRouter::StaticRouter(v.try_into()?),
      config::HttpRouterConfig::ProxyRouter(v) => v1::HttpRouter::ProxyRouter(v.try_into()?),
    })
  }
}

impl TryFrom<config::ProxyRouterConfig> for v1::ProxyRouter {
  type Error = ManifestError;
  fn try_from(value: config::ProxyRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      url: value.url.id().to_owned(),
      strip_path: value.strip_path,
      middleware: value.middleware.try_map_into()?,
    })
  }
}

impl TryFrom<config::StaticRouterConfig> for v1::StaticRouter {
  type Error = ManifestError;
  fn try_from(value: config::StaticRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      volume: value.volume.id().to_owned(),
      fallback: value.fallback,
      middleware: value.middleware.try_map_into()?,
      indexes: value.indexes,
    })
  }
}

impl TryFrom<config::RawRouterConfig> for v1::RawRouter {
  type Error = ManifestError;
  fn try_from(value: config::RawRouterConfig) -> Result<Self> {
    Ok(Self {
      path: value.path,
      codec: value.codec.map_into(),
      operation: value.operation.try_into()?,
      middleware: value.middleware.try_map_into()?,
    })
  }
}

impl TryFrom<config::RestRouterConfig> for v1::RestRouter {
  type Error = ManifestError;
  fn try_from(value: config::RestRouterConfig) -> Result<Self> {
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

  fn try_from(value: config::Tools) -> Result<Self> {
    Ok(Self { openapi: value.openapi })
  }
}

impl TryFrom<v1::Tools> for config::Tools {
  type Error = ManifestError;

  fn try_from(value: v1::Tools) -> Result<Self> {
    Ok(Self { openapi: value.openapi })
  }
}

impl TryFrom<v1::Route> for config::RestRoute {
  type Error = ManifestError;

  fn try_from(value: v1::Route) -> Result<Self> {
    Ok(Self {
      id: value.id,
      methods: value.methods.map_into(),
      sub_path: value.sub_path,
      operation: value.operation.try_into()?,
      description: value.description,
      summary: value.summary,
    })
  }
}

impl TryFrom<config::RestRoute> for v1::Route {
  type Error = ManifestError;

  fn try_from(value: config::RestRoute) -> Result<Self> {
    Ok(Self {
      id: value.id,
      methods: value.methods.map_into(),
      sub_path: value.sub_path,
      operation: value.operation.try_into()?,
      description: value.description,
      summary: value.summary,
    })
  }
}

impl TryFrom<v1::Info> for config::Info {
  type Error = ManifestError;

  fn try_from(value: v1::Info) -> Result<Self> {
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

  fn try_from(value: config::Info) -> Result<Self> {
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

  fn try_from(value: v1::Documentation) -> Result<Self> {
    Ok(Self {
      description: value.description,
      url: value.url,
    })
  }
}

impl TryFrom<config::Documentation> for v1::Documentation {
  type Error = ManifestError;

  fn try_from(value: config::Documentation) -> Result<Self> {
    Ok(Self {
      description: value.description,
      url: value.url,
    })
  }
}

impl TryFrom<v1::Contact> for config::Contact {
  type Error = ManifestError;

  fn try_from(value: v1::Contact) -> Result<Self> {
    Ok(Self {
      name: value.name,
      url: value.url,
      email: value.email,
    })
  }
}

impl TryFrom<config::Contact> for v1::Contact {
  type Error = ManifestError;

  fn try_from(value: config::Contact) -> Result<Self> {
    Ok(Self {
      name: value.name,
      url: value.url,
      email: value.email,
    })
  }
}

impl TryFrom<v1::License> for config::License {
  type Error = ManifestError;

  fn try_from(value: v1::License) -> Result<Self> {
    Ok(Self {
      name: value.name,
      url: value.url,
    })
  }
}

impl TryFrom<config::License> for v1::License {
  type Error = ManifestError;

  fn try_from(value: config::License) -> Result<Self> {
    Ok(Self {
      name: value.name,
      url: value.url,
    })
  }
}
