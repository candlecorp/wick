mod serialize;
mod wrapper;
pub(crate) use serialize::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use wick_config::config::components::SqlComponentConfig;
use wick_config::config::UrlResource;

use crate::Error;

pub(crate) async fn connect(_config: SqlComponentConfig, addr: &UrlResource) -> Result<PgPool, Error> {
  debug!(connect = %addr, "connecting to postgres");

  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&addr.to_string())
    .await
    .map_err(|e| Error::PostgresConnect(e.to_string()))?;
  Ok(pool)
}

#[cfg(test)]
mod integration_test {
  use anyhow::Result;
  use flow_component::{panic_callback, Component};
  use futures::StreamExt;
  use serde_json::json;
  use wick_config::config::components::SqlOperationDefinition;
  use wick_config::config::ResourceDefinition;
  use wick_config::HighLevelComponent;
  use wick_interface_types::{Field, TypeSignature};
  use wick_packet::{packet_stream, Invocation, Packet};

  use super::*;
  use crate::SqlXComponent;

  async fn init_pg_component() -> Result<SqlXComponent> {
    let docker_host = std::env::var("DOCKER_HOST").unwrap();
    let db_host = docker_host.split(':').next().unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let port = std::env::var("POSTGRES_PORT").unwrap();
    let user = "postgres";
    let db_name = "wick_test";

    let mut config = SqlComponentConfig {
      resource: "db".to_owned(),
      tls: false,
      operations: vec![],
    };
    let op = SqlOperationDefinition {
      name: "test".to_owned(),
      query: "select id,name from users where id = $1;".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::I32)],
      outputs: vec![Field::new("output", TypeSignature::Object)],
      arguments: vec!["input".to_owned()],
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource(
      "db",
      ResourceDefinition::Url(
        format!("postgres://{}:{}@{}:{}/{}", user, password, db_host, port, db_name)
          .try_into()
          .unwrap(),
      ),
    );

    let component = SqlXComponent::new();

    component.init(config, app_config.resolver()).await.unwrap();

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_pg_basic() -> Result<()> {
    let pg = init_pg_component().await?;
    let input = packet_stream!(("input", 1_u32));
    let inv = Invocation::test("postgres", "wick://__local__/test", None)?;
    let response = pg.handle(inv, input, None, panic_callback()).await.unwrap();
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
