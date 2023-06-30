mod serialize;
mod wrapper;
use sqlx::mssql::MssqlPoolOptions;
use sqlx::MssqlPool;
use url::Url;
use wick_config::config::components::SqlComponentConfig;

pub(crate) use self::serialize::*;
use crate::Error;

pub(crate) async fn connect(_config: SqlComponentConfig, addr: &Url) -> Result<MssqlPool, Error> {
  debug!(%addr, "connecting to mssql");

  let pool = MssqlPoolOptions::new()
    .max_connections(5)
    .connect(addr.as_ref())
    .await
    .map_err(|e| Error::MssqlConnect(e.to_string()))?;
  Ok(pool)
}

#[cfg(test)]
mod integration_test {
  use anyhow::Result;
  use flow_component::{panic_callback, Component};
  use futures::StreamExt;
  use serde_json::json;
  use wick_config::config::components::{
    ComponentConfig,
    SqlComponentConfigBuilder,
    SqlOperationDefinitionBuilder,
    SqlOperationKind,
  };
  use wick_config::config::ResourceDefinition;
  use wick_interface_types::{Field, Type};
  use wick_packet::{packet_stream, Invocation, Packet};

  use crate::SqlXComponent;

  async fn init_mssql_component() -> Result<SqlXComponent> {
    let docker_host = std::env::var("DOCKER_HOST").unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let db_host = docker_host.split(':').next().unwrap();
    let port = std::env::var("MSSQL_PORT").unwrap();
    let user = "SA";
    let db_name = "wick_test";

    let mut config = SqlComponentConfigBuilder::default()
      .resource("db")
      .tls(false)
      .build()
      .unwrap();
    let op = SqlOperationDefinitionBuilder::default()
      .name("test")
      .query("select id,name from users where id=$1;")
      .inputs([Field::new("input", Type::I32)])
      .outputs([Field::new("output", Type::Object)])
      .arguments(["input".to_owned()])
      .build()
      .unwrap();

    config.operations_mut().push(SqlOperationKind::Query(op));
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource(
      "db",
      ResourceDefinition::Url(
        format!("mssql://{}:{}@{}:{}/{}", user, password, db_host, port, db_name)
          .try_into()
          .unwrap(),
      ),
    );

    let component = SqlXComponent::new(config, None, None, &app_config.resolver())?;

    component.init().await.unwrap();

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_mssql_basic() -> Result<()> {
    let pg = init_mssql_component().await?;
    let input = packet_stream!(("input", 1_i32));
    let inv = Invocation::test("postgres", "wick://__local__/test", input, None)?;
    let response = pg.handle(inv, Default::default(), panic_callback()).await.unwrap();
    let packets: Vec<_> = response.collect().await;

    assert_eq!(
      packets,
      vec![
        Ok(Packet::encode("output", json!({"id":1_i32, "name":"Test User"}))),
        Ok(Packet::done("output"))
      ]
    );
    Ok(())
  }
}
