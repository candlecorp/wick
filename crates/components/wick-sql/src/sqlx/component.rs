use std::collections::HashMap;

use flow_component::ComponentError;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde_json::Value;
use sqlx::{PgPool, SqlitePool};
use url::Url;
use wick_config::config::components::{ComponentConfig, OperationConfig, SqlComponentConfig};
use wick_config::config::ErrorBehavior;
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{Field, Type};

use crate::common::sql_wrapper::ConvertedType;
use crate::common::{ClientConnection, Connection, DatabaseProvider};
use crate::sqlx::{postgres, sqlite};
use crate::{common, Error};

#[derive(Debug, Clone)]
enum CtxPool {
  Postgres(PgPool),
  SqlLite(SqlitePool),
}

impl CtxPool {
  fn run_query<'a, 'b>(&'a self, querystr: &'b str, args: Vec<ConvertedType>) -> BoxStream<'a, Result<Value, Error>>
  where
    'b: 'a,
  {
    match self {
      CtxPool::Postgres(c) => {
        let query = postgres::make_query(querystr, args);
        let stream = query.fetch(c).map(|res| res.map(postgres::SerMapRow::from)).map(|res| {
          res
            .map(|el| serde_json::to_value(el).unwrap_or(Value::Null))
            .map_err(|e| Error::Fetch(e.to_string()))
        });

        stream.boxed()
      }
      CtxPool::SqlLite(c) => {
        let query = sqlite::make_query(querystr, args);
        let stream = query.fetch(c).map(|res| res.map(sqlite::SerMapRow::from)).map(|res| {
          res
            .map(|el| serde_json::to_value(el).unwrap_or(Value::Null))
            .map_err(|e| Error::Fetch(e.to_string()))
        });

        stream.boxed()
      }
    }
  }

  async fn run_exec<'a, 'q>(&'a self, query: &'q str, args: Vec<ConvertedType>) -> Result<u64, Error>
  where
    'q: 'a,
  {
    let result = match self {
      CtxPool::Postgres(c) => {
        let query = postgres::make_query(query, args);
        query.execute(c).await.map(|r| r.rows_affected())
      }
      CtxPool::SqlLite(c) => {
        let query = sqlite::make_query(query, args);
        query.execute(c).await.map(|r| r.rows_affected())
      }
    };
    result.map_err(|e| Error::Exec(e.to_string()))
  }
}

#[derive(Clone)]
pub(crate) struct Context {
  db: CtxPool,
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").finish()
  }
}

impl Context {}

/// The SQLx component.
#[derive(Debug, Clone)]
#[must_use]
pub(crate) struct SqlXComponent {
  context: Context,
  prepared_queries: HashMap<String, String>,
}

impl SqlXComponent {
  /// Create a new SQLx component.
  pub(crate) async fn new(config: SqlComponentConfig, resolver: &Resolver) -> Result<Self, Error> {
    validate(&config, resolver)?;
    let url = common::convert_url_resource(resolver, config.resource())?;
    let context = init_context(&config, &url).await?;
    let mut queries = HashMap::new();
    trace!(count=%config.operations().len(), "preparing queries");
    for op in config.operations() {
      queries.insert(op.name().to_owned(), op.query().to_owned());
    }
    Ok(Self {
      context,
      prepared_queries: queries,
    })
  }
}

#[async_trait::async_trait]
impl DatabaseProvider for SqlXComponent {
  fn get_statement<'a>(&'a self, id: &'a str) -> Option<&'a str> {
    self.prepared_queries.get(id).map(|e| e.as_str())
  }

  async fn get_connection<'a, 'b>(&'a self) -> Result<Connection<'b>, Error>
  where
    'a: 'b,
  {
    Ok(Connection::new(Box::new(self.context.db.clone())))
  }
}

#[async_trait::async_trait]
impl ClientConnection for CtxPool {
  async fn finish(&mut self, _behavior: ErrorBehavior) -> Result<(), Error> {
    // todo
    Ok(())
  }

  async fn start(&mut self, _behavior: ErrorBehavior) -> Result<(), Error> {
    // todo
    Ok(())
  }

  async fn handle_error(&mut self, _e: Error, _behavior: ErrorBehavior) -> Result<(), Error> {
    // todo
    Ok(())
  }

  async fn exec(&mut self, stmt: String, bound_args: Vec<ConvertedType>) -> Result<u64, Error> {
    self.run_exec(&stmt, bound_args).await
  }

  async fn query<'a, 'b>(
    &'a mut self,
    stmt: &'a str,
    bound_args: Vec<ConvertedType>,
  ) -> Result<BoxStream<'b, Result<Value, Error>>, Error>
  where
    'a: 'b,
  {
    let stream = self.run_query(stmt.as_ref(), bound_args);
    Ok(stream.boxed())
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
      outputs.len() > 1 || outputs.len() == 1 && outputs[0] != Field::new("output", Type::Object)
    })
    .map(|op| op.name().to_owned())
    .collect();

  if !bad_ops.is_empty() {
    return Err(Error::InvalidOutput(bad_ops));
  }

  Ok(())
}

async fn init_client(config: &SqlComponentConfig, addr: &Url) -> Result<CtxPool, Error> {
  let pool = match addr.scheme() {
    "file" => CtxPool::SqlLite(
      sqlite::connect(
        config,
        Some(
          addr
            .to_file_path()
            .map_err(|_e| Error::SqliteConnect(format!("could not convert url {} to filepath", addr)))?
            .to_str()
            .unwrap(),
        ),
      )
      .await?,
    ),
    "postgres" => CtxPool::Postgres(postgres::connect(config, addr).await?),
    "sqlite" => {
      if addr.host() != Some(url::Host::Domain("memory")) {
        return Err(Error::SqliteScheme);
      }
      CtxPool::SqlLite(sqlite::connect(config, None).await?)
    }
    "mysql" => unimplemented!("MySql is not supported yet"),
    "mssql" => unreachable!(),
    s => return Err(Error::InvalidScheme(s.to_owned())),
  };
  debug!(%addr, "connected to db");
  Ok(pool)
}

async fn init_context(config: &SqlComponentConfig, addr: &Url) -> Result<Context, Error> {
  let client = init_client(config, addr).await?;
  let db = client;

  Ok(Context { db })
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_config::config::components::{
    SqlComponentConfigBuilder,
    SqlOperationDefinition,
    SqlQueryOperationDefinitionBuilder,
  };
  use wick_config::config::{ResourceDefinition, TcpPort};
  use wick_interface_types::{Field, Type};

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
    let op = SqlQueryOperationDefinitionBuilder::default()
      .name("test")
      .query("select * from users where user_id = $1;")
      .inputs([Field::new("input", Type::I32)])
      .outputs([Field::new("output", Type::String)])
      .arguments(["input".to_owned()])
      .build()
      .unwrap();

    config.operations_mut().push(SqlOperationDefinition::Query(op));
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new("0.0.0.0", 11111)));

    let result = validate(&config, &app_config.resolver());
    assert!(result.is_err());
    Ok(())
  }
}
