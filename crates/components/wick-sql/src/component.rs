use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::StreamExt;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use tracing::Span;
use url::Url;
use wick_config::config::components::{ComponentConfig, OperationConfig, SqlComponentConfig, SqlOperationKind};
use wick_config::config::{ErrorBehavior, Metadata};
use wick_config::Resolver;
use wick_interface_types::{ComponentSignature, Field, OperationSignatures, Type};
use wick_packet::{Invocation, Observer, Packet, PacketSender, PacketStream, RuntimeConfig};

use crate::common::{Connection, DatabaseProvider};
use crate::{common, Error};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DbKind {
  Mssql,
  Postgres,
  Sqlite,
}
#[derive(Clone)]
struct Client {
  inner: Arc<dyn DatabaseProvider + Send + Sync>,
}

impl Client {
  async fn new(
    url: &Url,
    config: &mut SqlComponentConfig,
    _metadata: Option<Metadata>,
    _root_config: Option<RuntimeConfig>, // TODO use this
    resolver: &Resolver,
  ) -> Result<Self, Error> {
    let client: Arc<dyn DatabaseProvider + Send + Sync> = match url.scheme() {
      "mssql" => {
        normalize_operations(config.operations_mut(), DbKind::Mssql);
        Arc::new(crate::mssql_tiberius::AzureSqlComponent::new(config.clone(), resolver).await?)
      }
      "postgres" => {
        normalize_operations(config.operations_mut(), DbKind::Postgres);
        Arc::new(crate::sqlx::SqlXComponent::new(config.clone(), resolver).await?)
      }
      "sqlite" => {
        normalize_operations(config.operations_mut(), DbKind::Sqlite);

        Arc::new(crate::sqlx::SqlXComponent::new(config.clone(), resolver).await?)
      }

      _ => {
        return Err(Error::InvalidScheme(url.scheme().to_owned()));
      }
    };

    Ok(Self { inner: client })
  }

  fn inner(&self) -> &Arc<dyn DatabaseProvider + Sync + Send> {
    &self.inner
  }
}

#[async_trait::async_trait]
impl DatabaseProvider for Client {
  fn get_statement<'a>(&'a self, id: &'a str) -> Option<&'a str> {
    self.inner().get_statement(id)
  }

  async fn get_connection<'a, 'b>(&'a self) -> Result<Connection<'b>, Error>
  where
    'a: 'b,
  {
    self.inner().get_connection().await
  }
}
// pub(crate) struct Transaction<'a>(Arc<dyn ClientConnection + Sync + Send + 'a>);

// impl<'a> Transaction<'a> {
//   fn new(conn: Arc<dyn ClientConnection + Sync + Send + 'a>) -> Self {
//     Self(conn)
//   }
//   fn end_transaction(&self) -> BoxFuture<Result<(), Error>> {
//     Box::pin(async move { Ok(()) })
//   }
// }

// #[async_trait::async_trait]
// impl<'a> ClientConnection for Transaction<'a> {
//   async fn query(&mut self, stmt: &str, bound_args: Vec<SqlWrapper>) -> Result<BoxStream<Result<Value, Error>>, Error> {
//     self.0.query(stmt, bound_args).await
//   }
//   async fn exec(&mut self, stmt: &str, bound_args: Vec<SqlWrapper>) -> Result<(), Error> {
//     self.0.exec(stmt, bound_args).await
//   }

//   async fn handle_error(&mut self, e: Error, behavior: ErrorBehavior) -> Result<(), Error> {
//     self.0.handle_error(e, behavior).await
//   }

//   async fn finish(&mut self) -> Result<(), Error> {
//     todo!()
//   }
// }

/// The Azure SQL Wick component.
#[derive(Clone)]
#[must_use]
pub struct SqlComponent {
  provider: Client,
  signature: Arc<ComponentSignature>,
  url: Url,
  config: SqlComponentConfig,
  root_config: Option<RuntimeConfig>,
}

impl std::fmt::Debug for SqlComponent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SqlComponent")
      .field("signature", &self.signature)
      .field("url", &self.url)
      .field("config", &self.config)
      .field("root_config", &self.root_config)
      .finish()
  }
}

impl SqlComponent {
  /// Instantiate a new Azure SQL component.
  pub async fn new(
    mut config: SqlComponentConfig,
    root_config: Option<RuntimeConfig>,
    metadata: Option<Metadata>,
    resolver: &Resolver,
  ) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let sig = common::gen_signature(
      "wick/component/sql",
      config.operation_signatures(),
      config.config(),
      &metadata,
    )?;

    let url = common::convert_url_resource(resolver, config.resource())?;

    validate(&config, resolver)?;
    let provider = Client::new(&url, &mut config, metadata, root_config.clone(), resolver).await?;

    Ok(Self {
      provider,
      signature: Arc::new(sig),
      url,
      root_config,
      config,
    })
  }
}

impl Component for SqlComponent {
  fn handle(
    &self,
    mut invocation: Invocation,
    _data: Option<RuntimeConfig>, // TODO: this needs to be used
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let client = self.provider.clone();
    let opdef = self
      .config
      .get_operation(invocation.target.operation_id())
      .ok_or_else(|| Error::MissingOperation(invocation.target.operation_id().to_owned()))
      .cloned();

    Box::pin(async move {
      let opdef = opdef?;
      let stmt = client.get_statement(opdef.name()).unwrap().to_owned();

      let input_names: Vec<_> = opdef.inputs().iter().map(|i| i.name.clone()).collect();
      let input_streams = wick_packet::split_stream(invocation.eject_stream(), input_names);
      let (tx, rx) = invocation.make_response();
      tokio::spawn(async move {
        let start = SystemTime::now();
        let span = invocation.span.clone();
        if let Err(e) = handle_call(&client, opdef, input_streams, tx.clone(), &stmt, span).await {
          invocation.trace(|| {
            error!(error = %e, "error in sql operation");
          });
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        }
        let _ = tx.send(Packet::done("output"));
        let duration = SystemTime::now().duration_since(start).unwrap();
        invocation.trace(|| {
          debug!(?duration, target=%invocation.target,"mssql operation complete");
        });
      });

      Ok(rx)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
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

async fn handle_call<'a, 'b>(
  client: &'a Client,
  opdef: SqlOperationKind,
  input_streams: Vec<PacketStream>,
  tx: PacketSender,
  stmt: &'b str,
  span: Span,
) -> Result<(), Error>
where
  'b: 'a,
{
  let error_behavior = opdef.on_error();
  let mut connection = match error_behavior {
    ErrorBehavior::Commit | ErrorBehavior::Rollback => client.get_connection().await?, // TODO make transaction
    _ => client.get_connection().await?,
  };

  let result = handle_stream(&mut connection, opdef, input_streams, tx, stmt, span).await;
  if let Err(e) = result {
    let err = Error::OperationFailed(e.to_string());
    connection.handle_error(e, error_behavior).await?;
    return Err(err);
  }
  connection.finish().await?;

  Ok(())
}

async fn handle_stream<'a, 'b, 'c>(
  connection: &'a mut Connection<'c>,
  opdef: SqlOperationKind,
  mut input_streams: Vec<PacketStream>,
  tx: PacketSender,
  stmt: &'b str,
  span: Span,
) -> Result<(), Error>
where
  'b: 'a,
{
  span.in_scope(|| debug!(stmt = %stmt, "preparing query for stream"));
  'outer: loop {
    let mut incoming_packets = Vec::new();

    for input in &mut input_streams {
      let packet = input.next().await;

      incoming_packets.push(packet);
    }

    let num_done = incoming_packets.iter().filter(|r| r.is_none()).count();
    if num_done > 0 {
      if num_done != opdef.inputs().len() {
        return Err(Error::MissingInput);
      }
      break 'outer;
    }
    let incoming_packets = incoming_packets.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();

    if let Some(Err(e)) = incoming_packets.iter().find(|r| r.is_err()) {
      return Err(Error::ComponentError(e.clone()));
    }
    let fields = opdef.inputs();
    let mut type_wrappers = Vec::new();

    for packet in incoming_packets {
      let packet = packet.unwrap();
      if packet.is_done() {
        break 'outer;
      }
      if packet.is_open_bracket() || packet.is_close_bracket() {
        let _ = tx.send(packet.set_port("output"));
        continue 'outer;
      }
      let ty = fields.iter().find(|f| f.name() == packet.port()).unwrap().ty().clone();
      type_wrappers.push((ty, packet));
    }

    let start = SystemTime::now();
    let result = match &opdef {
      SqlOperationKind::Query(_) => {
        query(connection, tx.clone(), opdef.clone(), type_wrappers, stmt, span.clone()).await
      }
      SqlOperationKind::Exec(_) => exec(connection, tx.clone(), opdef.clone(), type_wrappers, stmt, span.clone()).await,
    };
    let duration = SystemTime::now().duration_since(start).unwrap();

    span.in_scope(|| debug!(μs = duration.as_micros(), "executed query"));

    if let Err(e) = result {
      if opdef.on_error() == ErrorBehavior::Ignore {
        let _ = tx.send(Packet::err("output", e.to_string()));
      } else {
        return Err(Error::OperationFailed(e.to_string()));
      }
    };

    if opdef.inputs().len() == 0 {
      break 'outer;
    }
  }
  Ok(())
}

async fn query<'a, 'b, 'c>(
  client: &'a mut Connection<'c>,
  tx: PacketSender,
  def: SqlOperationKind,
  args: Vec<(Type, Packet)>,
  stmt: &'b str,
  _span: Span,
) -> Result<Duration, Error>
where
  'b: 'a,
{
  let start = SystemTime::now();

  let bound_args = common::bind_args(def.arguments(), &args)?;

  let mut rows = client.query(stmt, bound_args).await?;

  while let Some(row) = rows.next().await {
    let _ = match row {
      Ok(row) => tx.send(Packet::encode("output", row)),
      Err(e) => tx.send(Packet::err("output", e.to_string())),
    };
  }

  let duration = SystemTime::now().duration_since(start).unwrap();

  Ok(duration)
}

async fn exec<'a, 'b, 'c>(
  connection: &'a mut Connection<'c>,
  tx: PacketSender,
  def: SqlOperationKind,
  args: Vec<(Type, Packet)>,
  stmt: &'b str,
  _span: Span,
) -> Result<Duration, Error>
where
  'b: 'a,
{
  let start = SystemTime::now();

  let bound_args = common::bind_args(def.arguments(), &args)?;
  let packet = match connection.exec(stmt.to_owned(), bound_args).await {
    Ok(num) => Packet::encode("output", num),
    Err(err) => Packet::err("output", err.to_string()),
  };

  let _ = tx.send(packet);

  let duration = SystemTime::now().duration_since(start).unwrap();

  Ok(duration)
}

static POSITIONAL_ARGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$(?<id>\d+)\b").unwrap());
static WICK_ID_ARGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{(?<id>\w+)\}").unwrap());

fn normalize_operations(ops: &mut Vec<SqlOperationKind>, db: DbKind) {
  for operations in ops {
    match operations {
      wick_config::config::components::SqlOperationKind::Query(ref mut op) => {
        let (mut query, args) = normalize_inline_ids(op.query(), op.arguments().to_vec());
        if db == DbKind::Mssql {
          query = normalize_mssql_query(query);
        }
        let query = query.to_string();
        op.set_query(query);
        op.set_arguments(args);
      }
      wick_config::config::components::SqlOperationKind::Exec(ref mut op) => {
        let (mut query, args) = normalize_inline_ids(op.exec(), op.arguments().to_vec());
        if db == DbKind::Mssql {
          query = normalize_mssql_query(query);
        }
        let query = query.to_string();
        op.set_exec(query);
        op.set_arguments(args);
      }
    };
  }
}

// This translates `${id}` to positional `$1` arguments.
fn normalize_inline_ids(orig_query: &str, mut orig_args: Vec<String>) -> (Cow<str>, Vec<String>) {
  if orig_query.contains('$') {
    // replace all instances of ${id} with @p1, @p2, etc and append the id to the args

    let mut id_map: HashMap<String, usize> = orig_args
      .iter()
      .enumerate()
      .map(|(i, id)| (id.clone(), i + 1))
      .collect();

    let captures = WICK_ID_ARGS.captures_iter(orig_query);
    for id in captures {
      let id = id.name("id").unwrap().as_str().to_owned();
      if !id_map.contains_key(&id) {
        id_map.insert(id.clone(), id_map.len() + 1);
        orig_args.push(id.clone());
      }
    }

    let normalized = WICK_ID_ARGS.replace_all(orig_query, |cap: &Captures| {
      let id = cap.name("id").unwrap().as_str();
      let id = id_map.get(id).unwrap();
      format!("${}", id)
    });
    debug!(%orig_query,%normalized, "sql:mssql:normalized query");
    (normalized, orig_args)
  } else {
    (Cow::Borrowed(orig_query), orig_args)
  }
}

// This translates `$1..$n` to `@p1..@pn` to be compatible with Tiberius.
fn normalize_mssql_query(original: Cow<str>) -> Cow<str> {
  if original.contains('$') {
    let normalized = POSITIONAL_ARGS.replace_all(&original, "@p${id}");
    debug!(%original,%normalized, "sql:mssql:normalized query");
    Cow::Owned(normalized.to_string())
  } else {
    original
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_mssql_query_normalization() -> Result<()> {
    let query = "select id,name from users where id=$1;";
    let expected = "select id,name from users where id=@p1;";
    let actual = normalize_mssql_query(Cow::Borrowed(query));
    assert_eq!(actual, expected);

    Ok(())
  }

  #[rstest::rstest]
  #[case("select id,name from users where id=${id};",[],"select id,name from users where id=$1;",["id"])]
  #[case("select id,name from users where email=$1, id=${id};",["email"],"select id,name from users where email=$1, id=$2;",["email","id"])]
  #[case("select id,name from users where email=$1, id=${id}, email=${email};",["email"],"select id,name from users where email=$1, id=$2, email=$1;",["email","id"])]
  #[case("select id,name from users where id=${id}, id2=${id}, id3=${id};",[],"select id,name from users where id=$1, id2=$1, id3=$1;",["id"])]
  fn test_inline_id_normalization<const K: usize, const U: usize>(
    #[case] orig_query: &str,
    #[case] orig_args: [&str; K],
    #[case] expected_query: &str,
    #[case] expected_args: [&str; U],
  ) -> Result<()> {
    let (actual, actual_args) =
      normalize_inline_ids(orig_query, orig_args.iter().copied().map(|s| s.to_owned()).collect());
    let expected_args = expected_args.iter().map(|s| s.to_owned()).collect::<Vec<_>>();
    assert_eq!(actual, expected_query);
    assert_eq!(actual_args, expected_args);

    Ok(())
  }
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

  use super::SqlComponent;

  async fn init_mssql_component() -> Result<SqlComponent> {
    let docker_host = std::env::var("TEST_HOST").unwrap();
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

    let mut component = SqlComponent::new(config, None, None, &app_config.resolver()).await?;

    component.init().await?;

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_mssql_basic() -> Result<()> {
    let db = init_mssql_component().await?;
    let input = packet_stream!(("input", 1_i32));
    let inv = Invocation::test("mssql", "wick://__local__/test", input, None)?;
    let response = db.handle(inv, Default::default(), panic_callback()).await.unwrap();
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

  async fn init_pg_component() -> Result<SqlComponent> {
    let docker_host = std::env::var("TEST_HOST").unwrap();
    let db_host = docker_host.split(':').next().unwrap();
    let password = std::env::var("TEST_PASSWORD").unwrap();
    let port = std::env::var("POSTGRES_PORT").unwrap();
    let user = "postgres";
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
        format!("postgres://{}:{}@{}:{}/{}", user, password, db_host, port, db_name)
          .try_into()
          .unwrap(),
      ),
    );

    let mut component = SqlComponent::new(config, None, None, &app_config.resolver()).await?;

    component.init().await.unwrap();

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_pg_basic() -> Result<()> {
    let pg = init_pg_component().await?;
    let input = packet_stream!(("input", 1_u32));
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

  async fn init_sqlite_component() -> Result<SqlComponent> {
    let db = std::env::var("SQLITE_DB").unwrap();

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
      ResourceDefinition::Url(format!("sqlite://{}", db).try_into().unwrap()),
    );

    let mut component = SqlComponent::new(config, None, None, &app_config.resolver()).await?;

    component.init().await.unwrap();

    Ok(component)
  }

  #[test_logger::test(tokio::test)]
  async fn test_sqlite_basic() -> Result<()> {
    let pg = init_sqlite_component().await?;
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
