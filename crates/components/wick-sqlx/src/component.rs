use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::stream::BoxStream;
use futures::StreamExt;
use serde_json::Value;
use sqlx::{MssqlPool, PgPool};
use wick_config::config::components::{SqlComponentConfig, SqlOperationDefinition};
use wick_config::config::{OwnedConfigurationItem, TcpPort};
use wick_config::{HighLevelComponent, Resolver};
use wick_interface_types::{component, ComponentSignature, Field, TypeSignature};
use wick_packet::{FluxChannel, Invocation, Observer, Packet, PacketStream, StreamMap, TypeWrapper};
use wick_rpc::RpcHandler;

use crate::error::Error;
use crate::mssql::SerMapMssqlRow;
use crate::postgres::SerMapPgRow;
use crate::sql_wrapper::SqlWrapper;
use crate::{mssql, postgres};

#[derive(Debug, Clone)]
enum CtxPool {
  Postgres(PgPool),
  MsSql(MssqlPool),
}

impl CtxPool {
  fn fetch<'a, 'b>(&'a self, query: &'b str, args: Vec<SqlWrapper>) -> BoxStream<'a, Result<Value, Error>>
  where
    'b: 'a,
  {
    match self {
      CtxPool::Postgres(c) => {
        let mut query = sqlx::query(query);
        for arg in args {
          trace!(?arg, "binding arg");
          query = query.bind(arg);
        }
        let a = query.fetch(c);

        let b = a.map(|a| a.map(SerMapPgRow::from));
        let c = b.map(|a| {
          a.map(|a| serde_json::to_value(a).unwrap())
            .map_err(|e| Error::Fetch(e.to_string()))
        });
        c.boxed()
      }
      CtxPool::MsSql(c) => {
        let mut query = sqlx::query(query);
        for arg in args {
          trace!(?arg, "binding arg");
          query = query.bind(arg);
        }
        let a = query.fetch(c);
        let b = a.map(|a| a.map(SerMapMssqlRow::from));
        let c = b.map(|a| {
          a.map(|a| serde_json::to_value(a).unwrap())
            .map_err(|e| Error::Fetch(e.to_string()))
        });
        c.boxed()
      }
    }
  }
}

#[derive()]
pub(crate) struct Context {
  db: CtxPool,
  config: SqlComponentConfig,
  queries: HashMap<String, Arc<(String, String)>>,
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context")
      .field("config", &self.config)
      .field("queries", &self.queries.keys())
      .finish()
  }
}

impl Context {}

#[derive(Debug, Clone)]
#[must_use]
pub struct SqlXComponent {
  context: Arc<tokio::sync::Mutex<Option<Context>>>,
  signature: Arc<ComponentSignature>,
}

impl Default for SqlXComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl SqlXComponent {
  pub fn new() -> Self {
    let sig = component! {
      name: "wick-postgres",
      version: option_env!("CARGO_PKG_VERSION").unwrap(),
      operations: {}
    };
    Self {
      context: Arc::new(tokio::sync::Mutex::new(None)),
      signature: Arc::new(sig),
    }
  }
}

impl Component for SqlXComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<Value>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
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
            for input in &mut inputs {
              let mut results = Vec::new();
              results.push(input.next().await);
              let num_done = results.iter().filter(|r| r.is_none()).count();
              if num_done > 0 {
                if num_done != opdef.inputs.len() {
                  let _ = tx.send(Packet::component_error("Missing input"));
                }
                break 'outer;
              }
              let results = results.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
              if let Some(Err(e)) = results.iter().find(|r| r.is_err()) {
                let _ = tx.send(Packet::component_error(e.to_string()));
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
      Err(ComponentError::message("DB not initialized"))
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}

impl HighLevelComponent for SqlXComponent {
  type Config = SqlComponentConfig;

  fn init(
    &self,
    config: Self::Config,
    resolver: Resolver,
  ) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), ComponentError>> + Send + 'static>> {
    let ctx = self.context.clone();
    let addr: TcpPort = resolver(&config.resource).unwrap().try_resource().unwrap().into();
    let init_context = init_context(config, addr);

    Box::pin(async move {
      let new_ctx = init_context.await?;

      ctx.lock().await.replace(new_ctx);

      Ok(())
    })
  }

  fn validate(&self, config: &Self::Config, resolver: Resolver) -> Result<(), ComponentError> {
    Ok(validate(config, &resolver)?)
  }
}

fn validate(
  config: &SqlComponentConfig,
  _resolver: &impl Fn(&str) -> Option<OwnedConfigurationItem>,
) -> Result<(), Error> {
  let bad_ops: Vec<_> = config
    .operations
    .iter()
    .filter(|op| {
      op.outputs.len() > 1 || op.outputs.len() == 1 && op.outputs[0] != Field::new("output", TypeSignature::Object)
    })
    .map(|op| op.name.clone())
    .collect();

  if !bad_ops.is_empty() {
    return Err(Error::InvalidOutput(bad_ops));
  }

  Ok(())
}

async fn init_client(config: SqlComponentConfig, addr: TcpPort) -> Result<CtxPool, Error> {
  let pool = match config.vendor {
    wick_config::config::components::DatabaseKind::MsSql => CtxPool::MsSql(mssql::connect(config, &addr).await?),
    wick_config::config::components::DatabaseKind::Postgres => {
      CtxPool::Postgres(postgres::connect(config, &addr).await?)
    }
    wick_config::config::components::DatabaseKind::Mysql => todo!(),
    wick_config::config::components::DatabaseKind::Sqlite => todo!(),
  };
  debug!(addr=%addr.address(), "connected to db");
  Ok(pool)
}

async fn init_context(config: SqlComponentConfig, addr: TcpPort) -> Result<Context, Error> {
  let client = init_client(config.clone(), addr).await?;
  let mut queries = HashMap::new();
  trace!(count=%config.operations.len(), "preparing queries");
  for op in &config.operations {
    // let query: Query<Postgres, _> = sqlx::query(&op.query);
    // TODO: this is a hack to during the sqlx transition and this needs to support prepared queries properly.
    queries.insert(op.name.clone(), Arc::new((op.query.clone(), op.query.clone())));
    trace!(query=%op.query, "prepared query");
  }

  let db = client;

  Ok(Context {
    db,
    config: config.clone(),
    queries,
  })
}

impl RpcHandler for SqlXComponent {}

async fn exec(
  client: CtxPool,
  tx: FluxChannel<Packet, wick_packet::Error>,
  def: SqlOperationDefinition,
  args: Vec<(TypeSignature, Packet)>,
  stmt: Arc<(String, String)>,
) -> Result<(), Error> {
  debug!(stmt = %stmt.0, "executing postgres query");
  let input_list: Vec<_> = def.inputs.iter().map(|i| i.name.clone()).collect();

  let values = args
    .into_iter()
    .map(|(ty, r)| r.deserialize_into(ty))
    .collect::<Result<Vec<TypeWrapper>, wick_packet::Error>>();

  if let Err(e) = values {
    let _ = tx.send(Packet::component_error(e.to_string()));
    return Err(Error::Prepare(e.to_string()));
  }
  let values = values.unwrap();
  #[allow(trivial_casts)]
  let args = def
    .arguments
    .iter()
    .map(|a| input_list.iter().position(|i| i == a).unwrap())
    .map(|i| SqlWrapper(values[i].clone()))
    .collect::<Vec<_>>();

  // let mut query = sqlx::query(&stmt.1);
  let mut result = client.fetch(&stmt.1, args);

  // pin_mut!(result);
  while let Some(row) = result.next().await {
    info!("got row");
    if let Err(e) = row {
      let _ = tx.send(Packet::component_error(e.to_string()));
      return Err(Error::Fetch(e.to_string()));
    }
    let row = row.unwrap();
    let packet = Packet::encode("output", row);
    let _ = tx.send(packet);
  }
  let _ = tx.send(Packet::done("output"));

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
  use wick_config::config::components::{DatabaseKind, SqlOperationDefinition};
  use wick_config::config::{ResourceDefinition, TcpPort};
  use wick_interface_types::{Field, TypeSignature};

  use super::*;

  #[test]
  fn test_component() {
    fn is_send_sync<T: Sync>() {}
    is_send_sync::<SqlXComponent>();
  }

  #[test_logger::test(test)]
  fn test_validate() -> Result<()> {
    let mut config = SqlComponentConfig {
      resource: "db".to_owned(),
      user: "postgres".to_owned(),
      password: "postgres".to_owned(),
      database: "testdb".to_owned(),
      vendor: DatabaseKind::Postgres,
      tls: false,
      operations: vec![],
    };
    let op = SqlOperationDefinition {
      name: "test".to_owned(),
      query: "select * from users where user_id = $1;".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::I32)],
      outputs: vec![Field::new("output", TypeSignature::String)],
      arguments: vec!["input".to_owned()],
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new("0.0.0.0", 11111)));

    let result = validate(&config, &app_config.resolver());
    assert_eq!(result, Err(Error::InvalidOutput(vec!["test".to_owned()])));
    Ok(())
  }
}
