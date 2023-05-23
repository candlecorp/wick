use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::stream::BoxStream;
use futures::StreamExt;
use parking_lot::Mutex;
use serde_json::Value;
use sqlx::{MssqlPool, PgPool};
use wick_config::config::components::{SqlComponentConfig, SqlOperationDefinition};
use wick_config::config::{Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{component, ComponentSignature, Field, TypeSignature};
use wick_packet::{FluxChannel, Invocation, Observer, OperationConfig, Packet, PacketStream};
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
  fn fetch<'a, 'q>(&'a self, query: &'q str, args: Vec<SqlWrapper>) -> BoxStream<'a, Result<Value, Error>>
  where
    'q: 'a,
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
          a.map(|a| serde_json::to_value(a).unwrap_or(Value::Null))
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
          a.map(|a| serde_json::to_value(a).unwrap_or(Value::Null))
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
  context: Arc<Mutex<Option<Context>>>,
  signature: Arc<ComponentSignature>,
  url_resource: UrlResource,
  config: SqlComponentConfig,
}

impl SqlXComponent {
  #[allow(clippy::needless_pass_by_value)]
  pub fn new(config: SqlComponentConfig, metadata: Metadata, resolver: &Resolver) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let mut sig = component! {
      name: "wick/component/sql",
      version: metadata.version(),
      operations: config.operation_signatures(),
    };
    // NOTE: remove this must change when db components support customized outputs.
    sig.operations.iter_mut().for_each(|op| {
      if !op.outputs.iter().any(|f| f.name == "output") {
        op.outputs.push(Field::new("output", TypeSignature::Object));
      }
    });

    let addr = resolver(config.resource())
      .ok_or_else(|| ComponentError::message(&format!("Could not resolve resource ID {}", config.resource())))
      .and_then(|r| r.try_resource().map_err(ComponentError::new))?;

    Ok(Self {
      context: Arc::new(Mutex::new(None)),
      signature: Arc::new(sig),
      url_resource: addr.into(),
      config,
    })
  }
}

impl Component for SqlXComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _data: Option<OperationConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let ctx = self.context.clone();

    Box::pin(async move {
      let (opdef, client, stmt) = match ctx.lock().as_ref() {
        Some(ctx) => {
          let opdef = ctx
            .config
            .operations()
            .iter()
            .find(|op| op.name() == invocation.target.operation_id())
            .unwrap()
            .clone();
          let client = ctx.db.clone();
          let stmt = ctx.queries.get(invocation.target.operation_id()).unwrap().clone();
          (opdef, client, stmt)
        }
        None => return Err(ComponentError::message("DB not initialized")),
      };

      let input_names: Vec<_> = opdef.inputs().iter().map(|i| i.name.clone()).collect();
      let (tx, rx) = invocation.make_response();
      let mut input_streams = wick_packet::split_stream(invocation.packets, input_names);
      tokio::spawn(async move {
        'outer: loop {
          let mut incoming_packets = Vec::new();
          for input in &mut input_streams {
            incoming_packets.push(input.next().await);
          }
          let num_done = incoming_packets.iter().filter(|r| r.is_none()).count();
          if num_done > 0 {
            if num_done != opdef.inputs().len() {
              let _ = tx.error(wick_packet::Error::component_error("Missing input"));
            }
            break 'outer;
          }
          let incoming_packets = incoming_packets.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();

          if let Some(Err(e)) = incoming_packets.iter().find(|r| r.is_err()) {
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            break 'outer;
          }
          let fields = opdef.inputs();
          let mut type_wrappers = Vec::new();

          for packet in incoming_packets {
            let packet = packet.unwrap();
            if packet.is_done() {
              break 'outer;
            }
            let ty = fields.iter().find(|f| f.name() == packet.port()).unwrap().ty().clone();
            type_wrappers.push((ty, packet));
          }

          if let Err(e) = exec(client.clone(), tx.clone(), opdef.clone(), type_wrappers, stmt.clone()).await {
            error!(error = %e, "error executing query");
          }
        }
      });

      Ok(rx)
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }

  fn init(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), ComponentError>> + Send + 'static>> {
    let ctx = self.context.clone();
    let addr = self.url_resource.clone();
    let config = self.config.clone();

    Box::pin(async move {
      let new_ctx = init_context(config, addr).await?;

      ctx.lock().replace(new_ctx);

      Ok(())
    })
  }
}

impl ConfigValidation for SqlXComponent {
  type Config = SqlComponentConfig;
  fn validate(config: &Self::Config, resolver: &Resolver) -> Result<(), ComponentError> {
    Ok(validate(config, resolver)?)
  }
}

fn validate(config: &SqlComponentConfig, _resolver: &Resolver) -> Result<(), Error> {
  let bad_ops: Vec<_> = config
    .operations()
    .iter()
    .filter(|op| {
      let outputs = op.outputs();
      outputs.len() > 1 || outputs.len() == 1 && outputs[0] != Field::new("output", TypeSignature::Object)
    })
    .map(|op| op.name().to_owned())
    .collect();

  if !bad_ops.is_empty() {
    return Err(Error::InvalidOutput(bad_ops));
  }

  Ok(())
}

async fn init_client(config: SqlComponentConfig, addr: UrlResource) -> Result<CtxPool, Error> {
  /*****************
   *
   * NOTE: This component was *supposed* to handle sql server (mssql) but sqlx was so buggy it
   * wouldn't work. The mssql logic is still in this crate for now, but it's not used and may be
   * removed in the future.
   *
   */
  let pool = match addr.scheme() {
    "mssql" => CtxPool::MsSql(mssql::connect(config, &addr).await?),
    "postgres" => CtxPool::Postgres(postgres::connect(config, &addr).await?),
    "mysql" => unimplemented!("MySql is not supported yet"),
    "sqllite" => unimplemented!("Sqllite is not supported yet"),
    s => return Err(Error::InvalidScheme(s.to_owned())),
  };
  debug!(addr=%addr.address(), "connected to db");
  Ok(pool)
}

async fn init_context(config: SqlComponentConfig, addr: UrlResource) -> Result<Context, Error> {
  let client = init_client(config.clone(), addr).await?;
  let mut queries = HashMap::new();
  trace!(count=%config.operations().len(), "preparing queries");
  for op in config.operations() {
    // let query: Query<Postgres, _> = sqlx::query(&op.query);
    // TODO: this is a hack to during the sqlx transition and this needs to support prepared queries properly.
    queries.insert(
      op.name().to_owned(),
      Arc::new((op.query().to_owned(), op.query().to_owned())),
    );
    trace!(query=%op.query(), "prepared query");
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
  debug!(stmt = %stmt.0, "executing query");

  let mut bound_args = Vec::new();
  for arg in def.arguments() {
    let (ty, packet) = args.iter().find(|(_, p)| p.port() == arg).cloned().unwrap();
    let wrapper = match packet
      .deserialize_into(ty.clone())
      .map_err(|e| Error::Prepare(e.to_string()))
    {
      Ok(type_wrapper) => SqlWrapper(type_wrapper),
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        return Err(Error::Prepare(e.to_string()));
      }
    };
    bound_args.push(wrapper);
  }

  let mut result = client.fetch(&stmt.1, bound_args);

  while let Some(row) = result.next().await {
    if let Err(e) = row {
      let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
      return Err(Error::Fetch(e.to_string()));
    }
    let row = row.unwrap();
    let packet = Packet::encode("output", row);
    let _ = tx.send(packet);
  }
  let _ = tx.send(Packet::done("output"));

  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_config::config::components::{SqlComponentConfigBuilder, SqlOperationDefinitionBuilder};
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
    let mut config = SqlComponentConfigBuilder::default()
      .resource("db")
      .tls(false)
      .build()
      .unwrap();
    let op = SqlOperationDefinitionBuilder::default()
      .name("test")
      .query("select * from users where user_id = $1;")
      .inputs([Field::new("input", TypeSignature::I32)])
      .outputs([Field::new("output", TypeSignature::String)])
      .arguments(["input".to_owned()])
      .build()
      .unwrap();

    config.operations_mut().push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new("0.0.0.0", 11111)));

    let result = validate(&config, &app_config.resolver());
    assert_eq!(result, Err(Error::InvalidOutput(vec!["test".to_owned()])));
    Ok(())
  }
}
