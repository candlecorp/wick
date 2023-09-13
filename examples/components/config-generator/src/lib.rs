mod wick {
  wick_component::wick_import!();
}
use wick::*;
use wick_config::v1::{
  AppConfiguration,
  HttpRouter,
  HttpTrigger,
  ResourceBinding,
  ResourceDefinition,
  TcpPort,
  TriggerDefinition,
  Volume,
  WickConfig,
};

fn build_static_config(app_name: &str, dir: String) -> WickConfig {
  let port_resource_id = "PORT".to_owned();
  let volume_resource_id = "DIR".to_owned();
  let config = AppConfiguration {
    name: app_name.to_owned(),
    resources: vec![
      ResourceBinding {
        name: port_resource_id.clone(),
        resource: ResourceDefinition::TcpPort(TcpPort {
          port: "{{ctx.env.HTTP_PORT}}".to_owned(),
          address: "0.0.0.0".to_owned(),
        }),
      },
      ResourceBinding {
        name: volume_resource_id.clone(),
        resource: ResourceDefinition::Volume(Volume { path: dir }),
      },
    ],
    triggers: vec![TriggerDefinition::HttpTrigger(HttpTrigger {
      resource: volume_resource_id.clone(),
      routers: vec![HttpRouter::StaticRouter(wick_config::v1::StaticRouter {
        path: "/".to_owned(),
        volume: volume_resource_id.clone(),
        middleware: None,
        fallback: None,
        indexes: true,
      })],
    })],
    ..Default::default()
  };
  WickConfig::AppConfiguration(config)
}

#[wick_component::operation(unary_simple)]
fn static_site(dir: String, ctx: Context<static_site::Config>) -> Result<String, anyhow::Error> {
  let config = build_static_config(&ctx.config.app_name, dir);
  let yaml = serde_yaml::to_string(&config)?;
  Ok(yaml)
}
