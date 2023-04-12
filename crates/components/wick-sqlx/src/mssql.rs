mod serialize;
mod wrapper;
pub(crate) use serialize::*;
use sqlx::mssql::MssqlPoolOptions;
use sqlx::MssqlPool;
use wick_config::config::components::SqlComponentConfig;
use wick_config::config::TcpPort;

use crate::Error;

pub(crate) async fn connect(config: SqlComponentConfig, addr: &TcpPort) -> Result<MssqlPool, Error> {
  let connect_string = format!(
    "mssql://{}:{}@{}/{}",
    config.user,
    config.password,
    addr.address(),
    config.database
  );
  debug!(connect = connect_string, "connecting to mssql");

  let pool = MssqlPoolOptions::new()
    .max_connections(5)
    .connect(&connect_string)
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
  use wick_config::config::components::{DatabaseKind, SqlOperationDefinition};
  use wick_config::config::{ResourceDefinition, TcpPort};
  use wick_config::HighLevelComponent;
  use wick_interface_types::{Field, TypeSignature};
  use wick_packet::{packet_stream, Invocation, Packet};

  use super::*;
  use crate::SqlXComponent;

  async fn init_mssql_component() -> Result<SqlXComponent> {
    let docker_host = std::env::var("DOCKER_HOST").unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let db_host = docker_host.split(':').next().unwrap();
    let port = std::env::var("MSSQL_PORT").unwrap();

    let mut config = SqlComponentConfig {
      resource: "db".to_owned(),
      user: "SA".to_owned(),
      password,
      database: "wick_test".to_owned(),
      vendor: DatabaseKind::MsSql,
      tls: false,
      operations: vec![],
    };
    let op = SqlOperationDefinition {
      name: "test".to_owned(),
      query: "select id,name from users where id=$1;".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::I32)],
      outputs: vec![Field::new("output", TypeSignature::Object)],
      arguments: vec!["input".to_owned()],
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource(
      "db",
      ResourceDefinition::TcpPort(TcpPort::new(db_host, port.parse().unwrap())),
    );

    let component = SqlXComponent::new();

    component.init(config, app_config.resolver()).await.unwrap();

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_mssql_basic() -> Result<()> {
    let pg = init_mssql_component().await?;
    let input = packet_stream!(("input", 1_i32));
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
