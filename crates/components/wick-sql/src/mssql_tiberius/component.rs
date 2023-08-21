use std::collections::HashMap;

use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use flow_component::ComponentError;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde_json::{Map, Value};
use tiberius::{Query, Row};
use url::Url;
use wick_config::config::components::{ComponentConfig, OperationConfig, SqlComponentConfig};
use wick_config::config::ErrorBehavior;
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{Field, Type};

use super::sql_wrapper::FromSqlWrapper;
use crate::common::sql_wrapper::ConvertedType;
use crate::common::{ClientConnection, Connection, DatabaseProvider};
use crate::{common, Error};

#[derive(Clone)]
pub(crate) struct Context {
  db: Pool<ConnectionManager>,
}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").finish()
  }
}

impl Context {
  async fn get<'a, 'b>(&'a self) -> Result<Connection<'b>, Error>
  where
    'a: 'b,
  {
    let conn = self.db.get().await.map_err(|e| Error::PoolConnection(e.to_string()))?;
    Ok(Connection::new(Box::new(conn)))
  }
}

#[async_trait::async_trait]
impl<'a> ClientConnection for PooledConnection<'a, ConnectionManager> {
  async fn finish(&mut self, _behavior: ErrorBehavior) -> Result<(), Error> {
    // todo
    Ok(())
  }

  async fn start(&mut self, behavior: ErrorBehavior) -> Result<(), Error> {
    match behavior {
      ErrorBehavior::Commit | ErrorBehavior::Rollback => {
        self.simple_query("BEGIN TRAN").await.map_err(|_| Error::TxStart)?;
      }
      _ => {}
    }
    Ok(())
  }

  async fn handle_error(&mut self, e: Error, behavior: ErrorBehavior) -> Result<(), Error> {
    match behavior {
      ErrorBehavior::Commit => {
        error!(error=%e, on_error=?behavior, "error in sql operation, committing transaction");
        self.simple_query("COMMIT").await.map_err(|_| Error::TxCommit)?;
      }
      ErrorBehavior::Rollback => {
        error!(error=%e, on_error=?behavior, "error in sql operation, rolling back transaction");
        self.simple_query("ROLLBACK").await.map_err(|_| Error::TxCommit)?;
      }
      _ => {}
    }
    Ok(())
  }

  async fn exec(&mut self, stmt: String, bound_args: Vec<ConvertedType>) -> Result<u64, Error> {
    let mut query = Query::new(stmt);

    for param in bound_args {
      query.bind(param);
    }

    query
      .execute(self)
      .await
      .map_err(|e| Error::Failed(e.to_string()))
      .map(|r| r.rows_affected()[0])
  }

  async fn query<'b, 'c>(
    &'b mut self,
    stmt: &'b str,
    bound_args: Vec<ConvertedType>,
  ) -> Result<BoxStream<'c, Result<Value, Error>>, Error>
  where
    'b: 'c,
  {
    let mut query = Query::new(stmt);

    for param in bound_args {
      query.bind(param);
    }

    let result = query.query(self).await.map_err(|e| Error::Failed(e.to_string()))?;

    Ok(
      result
        .filter(|row| futures::future::ready(!matches!(row, Ok(tiberius::QueryItem::Metadata(_)))))
        .map(|row| {
          row
            .map_err(|e| {
              tracing::Span::current().in_scope(|| tracing::error!(error=%e,"sql error in stream"));
              Error::QueryFailed
            })
            .and_then(|row| {
              row
                .into_row()
                .map_or_else(|| Err(Error::NoRow), |row| Ok(row_to_json(&row)))
            })
        })
        .boxed(),
    )
  }
}

/// The Azure SQL Wick component.
#[derive(Debug, Clone)]
#[must_use]
pub(crate) struct AzureSqlComponent {
  context: Context,
  prepared_queries: HashMap<String, String>,
}

impl AzureSqlComponent {
  /// Instantiate a new Azure SQL component.
  pub(crate) async fn new(config: SqlComponentConfig, resolver: &Resolver) -> Result<Self, Error> {
    validate(&config, resolver)?;

    let url = common::convert_url_resource(resolver, config.resource())?;
    let mut queries = HashMap::new();
    trace!(count=%config.operations().len(), "preparing queries");
    for op in config.operations() {
      queries.insert(op.name().to_owned(), op.query().to_owned());
    }
    let context = init_context(&config, url.clone()).await?;

    Ok(Self {
      context,
      prepared_queries: queries,
    })
  }
}

#[async_trait::async_trait]
impl DatabaseProvider for AzureSqlComponent {
  fn get_statement<'a>(&'a self, id: &'a str) -> Option<&'a str> {
    self.prepared_queries.get(id).map(|e| e.as_str())
  }

  async fn get_connection<'a, 'b>(&'a self) -> Result<Connection<'b>, Error>
  where
    'a: 'b,
  {
    self.context.get().await
  }
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

async fn init_client(config: &SqlComponentConfig, addr: Url) -> Result<Pool<ConnectionManager>, Error> {
  let pool = match addr.scheme() {
    "mssql" => super::mssql::connect(config, &addr).await?,
    s => return Err(Error::InvalidScheme(s.to_owned())),
  };
  debug!(%addr, "connected to db");
  Ok(pool)
}

async fn init_context(config: &SqlComponentConfig, addr: Url) -> Result<Context, Error> {
  let db = init_client(config, addr).await?;

  Ok(Context { db })
}

fn row_to_json(row: &Row) -> Value {
  let mut map: Map<String, Value> = Map::new();
  for col in row.columns() {
    let v = row.get::<'_, FromSqlWrapper, _>(col.name());
    map.insert(col.name().to_owned(), v.map_or(Value::Null, |v| v.0));
  }
  Value::Object(map)
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_config::config::components::{SqlComponentConfigBuilder, SqlOperationDefinitionBuilder, SqlOperationKind};
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

    config.operations_mut().push(SqlOperationKind::Query(op));
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource("db", ResourceDefinition::TcpPort(TcpPort::new("0.0.0.0", 11111)));

    let result = validate(&config, &app_config.resolver());
    assert!(result.is_err());
    Ok(())
  }
}
