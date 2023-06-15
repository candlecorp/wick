use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::{Future, StreamExt};
use parking_lot::Mutex;
use serde_json::{Map, Value};
use tiberius::{Query, Row};
use tracing::Span;
use wick_config::config::components::{SqlComponentConfig, SqlOperationDefinition};
use wick_config::config::{ErrorBehavior, Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{component, ComponentSignature, Field, Type};
use wick_packet::{FluxChannel, GenericConfig, Invocation, Observer, Packet, PacketSender, PacketStream};

use crate::error::Error;
use crate::mssql;
use crate::sql_wrapper::{FromSqlWrapper, SqlWrapper};

#[derive()]
pub(crate) struct Context {
  db: Pool<ConnectionManager>,
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
pub struct AzureSqlComponent {
  context: Arc<Mutex<Option<Context>>>,
  signature: Arc<ComponentSignature>,
  url_resource: UrlResource,
  config: SqlComponentConfig,
}

impl AzureSqlComponent {
  #[allow(clippy::needless_pass_by_value)]
  pub fn new(
    config: SqlComponentConfig,
    metadata: Option<Metadata>,
    resolver: &Resolver,
  ) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let mut sig = component! {
      name: "wick/component/sql",
      version: metadata.map(|v|v.version().to_owned()),
      operations: config.operation_signatures(),
    };

    // NOTE: remove this must change when db components support customized outputs.
    sig.operations.iter_mut().for_each(|op| {
      if !op.outputs.iter().any(|f| f.name == "output") {
        op.outputs.push(Field::new("output", Type::Object));
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

impl Component for AzureSqlComponent {
  fn handle(
    &self,
    mut invocation: Invocation,
    _data: Option<GenericConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let ctx = self.context.clone();

    Box::pin(async move {
      let (opdef, pool, stmt) = match ctx.lock().as_ref() {
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
      let input_streams = wick_packet::split_stream(invocation.eject_stream(), input_names);
      let (tx, rx) = invocation.make_response();
      tokio::spawn(async move {
        let start = SystemTime::now();
        let span = invocation.span.clone();
        if let Err(e) = handle_call(pool, opdef, input_streams, tx.clone(), stmt, span).await {
          invocation.trace(|| {
            error!(error = %e, "error in sql operation");
          });
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        }
        let _ = tx.send(Packet::done("output"));
        let duration = SystemTime::now().duration_since(start).unwrap();
        invocation.trace(|| {
          debug!(?duration, target=%invocation.target,"mssql operation complete");
        })
      });

      Ok(rx)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }

  fn init(&self) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ComponentError>> + Send + 'static>> {
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

async fn handle_call(
  pool: Pool<ConnectionManager>,
  opdef: SqlOperationDefinition,
  input_streams: Vec<PacketStream>,
  tx: PacketSender,
  stmt: Arc<(String, String)>,
  span: Span,
) -> Result<(), Error> {
  let mut client = pool.get().await.map_err(|e| Error::PoolConnection(e.to_string()))?;
  let error_behavior = *opdef.on_error();
  match error_behavior {
    ErrorBehavior::Commit | ErrorBehavior::Rollback => {
      client.simple_query("BEGIN TRAN").await.map_err(|_| Error::TxStart)?;
    }
    _ => {}
  }
  if let Err(e) = handle_stream(&mut client, opdef, input_streams, tx, stmt.clone(), span).await {
    match error_behavior {
      wick_config::config::ErrorBehavior::Commit => {
        client.simple_query("COMMIT").await.map_err(|_| Error::TxCommit)?;
      }
      wick_config::config::ErrorBehavior::Rollback => {
        client.simple_query("ROLLBACK").await.map_err(|_| Error::TxRollback)?;
      }
      _ => {}
    }
    return Err(Error::OperationFailed(e.to_string()));
  } else {
    match error_behavior {
      ErrorBehavior::Commit | ErrorBehavior::Rollback => {
        client.simple_query("COMMIT").await.map_err(|_| Error::TxCommit)?;
      }
      _ => {}
    }
  }
  Ok(())
}

async fn handle_stream(
  client: &mut PooledConnection<'_, ConnectionManager>,
  opdef: SqlOperationDefinition,
  mut input_streams: Vec<PacketStream>,
  tx: PacketSender,
  stmt: Arc<(String, String)>,
  span: Span,
) -> Result<(), Error> {
  'outer: loop {
    let mut incoming_packets = Vec::new();

    for input in &mut input_streams {
      incoming_packets.push(input.next().await);
    }

    let num_done = incoming_packets.iter().filter(|r| r.is_none()).count();
    if num_done > 0 {
      if num_done != opdef.inputs().len() {
        return Err(Error::MissingInput);
      }
    }
    let incoming_packets = incoming_packets.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();

    if let Some(Err(e)) = incoming_packets.iter().find(|r| r.is_err()) {
      return Err(Error::ComponentError(e.to_owned()));
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

    let _ = exec(
      client,
      tx.clone(),
      opdef.clone(),
      type_wrappers,
      stmt.clone(),
      span.clone(),
    )
    .await?;
  }
  Ok(())
}

async fn exec(
  client: &mut PooledConnection<'_, ConnectionManager>,
  tx: FluxChannel<Packet, wick_packet::Error>,
  def: SqlOperationDefinition,
  args: Vec<(Type, Packet)>,
  stmt: Arc<(String, String)>,
  span: Span,
) -> Result<Duration, Error> {
  let start = SystemTime::now();
  span.in_scope(|| trace!(stmt = %stmt.0, "executing query"));

  let mut bound_args: Vec<SqlWrapper> = Vec::new();
  for arg in def.arguments() {
    let (ty, packet) = args.iter().find(|(_, p)| p.port() == arg).cloned().unwrap();
    let wrapper = match packet
      .to_type_wrapper(ty.clone())
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

  #[allow(trivial_casts)]
  let mut query = Query::new(&stmt.1);
  for param in bound_args {
    query.bind(param);
  }

  let mut result = query.query(client).await.map_err(|e| Error::Failed(e.to_string()))?;

  while let Some(row) = result.next().await {
    if let Err(e) = row {
      let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
      return Err(Error::Fetch(e.to_string()));
    }
    let row = row.unwrap();
    if let Some(row) = row.into_row() {
      let packet = Packet::encode("output", row_to_json(&row));
      let _ = tx.send(packet);
    }
  }
  let duration = SystemTime::now().duration_since(start).unwrap();

  Ok(duration)
}

impl ConfigValidation for AzureSqlComponent {
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
      outputs.len() > 1 || outputs.len() == 1 && outputs[0] != Field::new("output", Type::Object)
    })
    .map(|op| op.name().to_owned())
    .collect();

  if !bad_ops.is_empty() {
    return Err(Error::InvalidOutput(bad_ops));
  }

  Ok(())
}

async fn init_client(config: SqlComponentConfig, addr: UrlResource) -> Result<Pool<ConnectionManager>, Error> {
  let pool = match addr.scheme() {
    "mssql" => mssql::connect(config, &addr).await?,
    "postgres" => unimplemented!("Use the sql component instead"),
    "mysql" => unimplemented!("Use the sql component instead"),
    "sqllite" => unimplemented!("Use the sql component instead"),
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

fn row_to_json(row: &Row) -> Value {
  let mut map: Map<String, Value> = Map::new();
  for col in row.columns() {
    let v = row.get::<'_, FromSqlWrapper, _>(col.name()).unwrap();
    map.insert(col.name().to_owned(), v.0);
  }
  Value::Object(map)
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_config::config::components::{SqlComponentConfigBuilder, SqlOperationDefinitionBuilder};
  use wick_config::config::{ResourceDefinition, TcpPort};
  use wick_interface_types::{Field, Type};

  use super::*;

  #[test]
  fn test_component() {
    fn is_send_sync<T: Sync>() {}
    is_send_sync::<AzureSqlComponent>();
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
      .inputs([Field::new("input", Type::I32)])
      .outputs([Field::new("output", Type::String)])
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
