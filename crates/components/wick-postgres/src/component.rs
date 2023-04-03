use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BufMut;
use futures::{pin_mut, Future, StreamExt, TryStreamExt};
use openssl::ssl::{SslConnector, SslMethod};
use parking_lot::Mutex;
use postgres_openssl::{MakeTlsConnector, TlsConnector};
use serde_json::{Number, Value};
use tokio::net::{TcpSocket, TcpStream};
use tokio_postgres::tls::{MakeTlsConnect, NoTlsStream, TlsStream};
use tokio_postgres::types::{accepts, to_sql_checked, FromSql, FromSqlOwned, IsNull, ToSql, Type};
use tokio_postgres::{Client, Config, Connection, NoTls, Statement};
use wick_config::config::components::{PostgresComponent, PostgresOperationDefinition};
use wick_config::config::{AppConfiguration, ConfigurationItem, TcpPort, UdpPort};
use wick_interface_types::{component, ComponentSignature, Field, HostedType, TypeSignature};
use wick_packet::{FluxChannel, Invocation, Observer, Packet, PacketPayload, PacketStream, StreamMap, TypeWrapper};
use wick_rpc::error::RpcError;
use wick_rpc::{dispatch, BoxFuture, RpcHandler, RpcResult};

use crate::conversions::postgres_row_to_json_value;
use crate::to_sql_wrapper::SqlWrapper;
use crate::{Error, NativeComponent, NativeComponentError};

#[derive()]
pub(crate) struct Context {
  db: Arc<Client>,
  config: PostgresComponent,
  queries: HashMap<String, Arc<(String, Statement)>>,
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context")
      .field("db", &self.db)
      .field("config", &self.config)
      .field("queries", &self.queries.keys())
      .finish()
  }
}

impl Context {}

#[derive(Debug, Clone)]
#[must_use]
pub struct Component {
  context: Arc<tokio::sync::Mutex<Option<Context>>>,
  signature: Arc<Mutex<ComponentSignature>>,
}

impl Default for Component {
  fn default() -> Self {
    Self::new()
  }
}

impl Component {
  pub fn new() -> Self {
    let sig = component! {
      name: "wick-postgres",
      version: option_env!("CARGO_PKG_VERSION").unwrap(),
      operations: {}
    };
    Self {
      context: Arc::new(tokio::sync::Mutex::new(None)),
      signature: Arc::new(Mutex::new(sig)),
    }
  }
}

impl NativeComponent for Component {
  type Config = PostgresComponent;
  fn init(
    &self,
    config: Self::Config,
    app_config: AppConfiguration,
  ) -> Pin<Box<dyn Future<Output = Result<(), NativeComponentError>> + Send>> {
    let ctx = self.context.clone();
    let addr: TcpPort = app_config
      .resolve_binding(&config.resource)
      .unwrap()
      .try_resource()
      .unwrap()
      .clone()
      .into();

    Box::pin(async move {
      let new_ctx = init_context(config, addr).await?;

      let lock = ctx.lock().await.replace(new_ctx);

      Ok(())
    })
  }
}

fn validate(config: &PostgresComponent, app_config: &AppConfiguration) -> Result<(), NativeComponentError> {
  let bad_ops: Vec<_> = config
    .operations
    .iter()
    .filter(|op| {
      op.outputs.len() > 1 || op.outputs.len() == 1 && op.outputs[0] != Field::new("output", TypeSignature::Object)
    })
    .map(|op| op.name.clone())
    .collect();

  if !bad_ops.is_empty() {
    return Err(NativeComponentError::InvalidOutput(bad_ops));
  }

  Ok(())
}

async fn init_client(config: PostgresComponent, addr: TcpPort) -> Result<Client, NativeComponentError> {
  let mut client = Config::new();
  client.user(&config.user);
  client.password(&config.password);
  client.dbname(&config.database);
  let socket = TcpSocket::new_v4().unwrap();
  let addr = SocketAddr::new(addr.address.parse().unwrap(), addr.port);
  let host = &addr.to_string();
  let stream = socket.connect(addr).await.unwrap();
  let client = if config.tls {
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let ctx = builder.build();
    let tls = TlsConnector::new(ctx.configure().unwrap(), host);
    let (client, conn) = client.connect_raw(stream, tls).await.unwrap();
    tokio::spawn(async move {
      if let Err(e) = conn.await {
        eprintln!("connection error: {}", e);
      }
    });

    client
  } else {
    let (client, conn) = client.connect_raw(stream, NoTls).await.unwrap();
    tokio::spawn(async move {
      if let Err(e) = conn.await {
        eprintln!("connection error: {}", e);
      }
    });

    client
  };
  debug!(host, "connected to postgres");
  Ok(client)
}

async fn init_context(config: PostgresComponent, addr: TcpPort) -> Result<Context, NativeComponentError> {
  let mut client = init_client(config.clone(), addr).await?;
  let mut queries = HashMap::new();
  trace!(count=%config.operations.len(), "preparing queries");
  for op in &config.operations {
    queries.insert(
      op.name.clone(),
      Arc::new((
        op.query.clone(),
        client
          .prepare(&op.query)
          .await
          .map_err(|_e| NativeComponentError::Temp)?,
      )),
    );
    trace!(query=%op.query, "prepared query");
  }

  let db = Arc::new(client);

  Ok(Context {
    db,
    config: config.clone(),
    queries,
  })
}

impl RpcHandler for Component {
  fn invoke(&self, invocation: Invocation, stream: PacketStream) -> BoxFuture<RpcResult<PacketStream>> {
    let target = invocation.target_url();
    trace!("stdlib invoke: {}", target);
    let ctx = self.context.clone();

    Box::pin(async move {
      let lock = ctx.lock().await;
      if let Some(ctx) = lock.as_ref() {
        let opdef = ctx
          .config
          .operations
          .iter()
          .find(|op| op.name == invocation.target.name())
          .unwrap()
          .clone();
        let client = ctx.db.clone();
        let stmt = ctx.queries.get(invocation.target.name()).unwrap().clone();

        let input_list: Vec<_> = opdef.inputs.iter().map(|i| i.name.clone()).collect();
        let mut inputs = fan_out(stream, &input_list);
        let (tx, rx) = PacketStream::new_channels();
        tokio::spawn(async move {
          'outer: loop {
            for mut input in &mut inputs {
              let mut results = Vec::new();
              results.push(input.next().await);
              let num_done = results.iter().filter(|r| r.is_none()).count();
              if num_done > 0 {
                if num_done != opdef.inputs.len() {
                  tx.send(Packet::component_error("Missing input"));
                }
                break 'outer;
              }
              let results = results.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
              if let Some(Err(e)) = results.iter().find(|r| r.is_err()) {
                tx.send(Packet::component_error(e.to_string()));
                break 'outer;
              }
              let results = results
                .into_iter()
                .enumerate()
                .map(|(i, r)| (opdef.inputs[i].ty.clone(), r.unwrap()))
                .collect::<Vec<_>>();
              if results.iter().any(|(_, r)| r.is_done()) {
                break 'outer;
              }

              if let Err(e) = exec(client.clone(), tx.clone(), opdef.clone(), results, stmt.clone()).await {
                error!(error = %e, "error executing postgres query");
              }
            }
          }
        });

        return Ok(rx);
      }
      Err(Box::new(RpcError::Component("DB not initialized".to_owned())))
    })
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let sig = self.signature.lock().clone();

    Ok(vec![HostedType::Component(sig)])
  }
}

async fn exec(
  client: Arc<Client>,
  tx: FluxChannel<Packet, wick_packet::Error>,
  def: PostgresOperationDefinition,
  results: Vec<(TypeSignature, Packet)>,
  stmt: Arc<(String, Statement)>,
) -> Result<(), NativeComponentError> {
  debug!(stmt = %stmt.0, "executing postgres query");
  let input_list: Vec<_> = def.inputs.iter().map(|i| i.name.clone()).collect();

  let values = results
    .into_iter()
    .map(|(ty, r)| r.deserialize_into(ty))
    .collect::<Result<Vec<TypeWrapper>, wick_packet::Error>>();

  if let Err(e) = values {
    tx.send(Packet::component_error(e.to_string()));
    return Err(NativeComponentError::Temp);
  }
  let values = values.unwrap();
  #[allow(trivial_casts)]
  let args = def
    .arguments
    .iter()
    .map(|a| input_list.iter().position(|i| i == a).unwrap())
    .map(|i| SqlWrapper(values[i].clone()))
    .collect::<Vec<_>>();

  let result = client.query_raw(&stmt.1, args).await;
  if let Err(e) = result {
    warn!(error = %e, "error executing postgres query");
    tx.send(Packet::component_error(e.to_string()));
    return Err(NativeComponentError::Temp);
  }
  let result = result.unwrap();
  pin_mut!(result);
  while let Some(row) = result.next().await {
    info!(?row, "row");
    if let Err(e) = row {
      tx.send(Packet::component_error(e.to_string()));
      return Err(NativeComponentError::Temp);
    }
    let row = row.unwrap();

    let value = postgres_row_to_json_value(&row).map_err(|e| {
      warn!(error = %e, "error converting postgres row to json");
      NativeComponentError::Temp
    })?;

    let packet = Packet::encode("output", value);
    tx.send(packet);
  }
  tx.send(Packet::done("output"));

  Ok(())
}

fn fan_out(mut stream: PacketStream, ports: &[String]) -> Vec<PacketStream> {
  let mut streams = StreamMap::default();
  let mut senders = HashMap::new();
  for port in ports {
    senders.insert(port.clone(), streams.init(port));
  }
  tokio::spawn(async move {
    while let Some(Ok(payload)) = stream.next().await {
      let sender = senders.get_mut(payload.port()).unwrap();
      if payload.is_done() {
        sender.complete();
        continue;
      }
      sender.send(payload).unwrap();
    }
  });
  ports.iter().map(|port| streams.take(port).unwrap()).collect()
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;
  use wick_config::config::components::PostgresOperationDefinition;
  use wick_config::config::{ComponentDefinition, HighLevelComponent, ResourceDefinition, TcpPort};
  use wick_interface_types::{Field, TypeSignature};
  use wick_packet::packet_stream;

  use super::*;

  fn is_send_sync<T>()
  where
    T: Sync,
  {
  }

  #[test]
  fn test_component() {
    is_send_sync::<SqlWrapper>();
  }

  async fn init_component() -> Result<Component> {
    let mut config = PostgresComponent {
      resource: "db".to_owned(),
      user: "postgres".to_owned(),
      password: "postgres".to_owned(),
      database: "testdb".to_owned(),
      tls: false,
      operations: vec![],
    };
    let op = PostgresOperationDefinition {
      name: "test".to_owned(),
      query: "select * from users where user_id = $1;".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::U32)],
      outputs: vec![Field::new("output", TypeSignature::Object)],
      arguments: vec!["input".to_owned()],
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new(5432, "0.0.0.0")));

    let component = Component::new();
    component.init(config, app_config).await?;

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_basic() -> Result<()> {
    let pg = init_component().await?;
    let input = packet_stream!(("input", 10101_u32));
    let inv = Invocation::test("postgres", "wick://__local__/test", None)?;
    let mut response = pg.invoke(inv, input).await.unwrap();
    let packets: Vec<_> = response.collect().await;

    assert_eq!(
      packets,
      vec![
        Ok(Packet::encode("output", json!({"user_id":10101, "username":"HELLO"}))),
        Ok(Packet::done("output"))
      ]
    );
    Ok(())
  }

  #[test_logger::test(test)]
  fn test_validate() -> Result<()> {
    let mut config = PostgresComponent {
      resource: "db".to_owned(),
      user: "postgres".to_owned(),
      password: "postgres".to_owned(),
      database: "testdb".to_owned(),
      tls: false,
      operations: vec![],
    };
    let op = PostgresOperationDefinition {
      name: "test".to_owned(),
      query: "select * from users where user_id = $1;".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::U32)],
      outputs: vec![Field::new("output", TypeSignature::String)],
      arguments: vec!["input".to_owned()],
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new(5432, "0.0.0.0")));
    // app_config.add_import(
    //   "mydb",
    //   ComponentDefinition::HighLevelComponent(HighLevelComponent::Postgres(config)),
    // );
    // println!("{}", app_config.into_v1_yaml()?);

    let result = validate(&config, &app_config);
    assert_eq!(
      result,
      Err(NativeComponentError::InvalidOutput(vec!["test".to_owned()]))
    );
    Ok(())
  }
}
