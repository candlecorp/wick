mod serialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, SqlitePool};
use url::Url;
use wick_config::config::components::SqlComponentConfig;

pub(crate) use self::serialize::*;
use crate::common::sql_wrapper::ConvertedType;
use crate::Error;

pub(crate) async fn connect(_config: &SqlComponentConfig, addr: &Url) -> Result<SqlitePool, Error> {
  debug!(%addr, "connecting to sqlite");

  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(addr.as_ref())
    .await
    .map_err(|e| Error::SqliteConnect(e.to_string()))?;
  Ok(pool)
}

pub(crate) fn make_query(
  sql: &str,
  args: Vec<ConvertedType>,
) -> sqlx::query::Query<'_, Sqlite, <Sqlite as sqlx::database::HasArguments>::Arguments> {
  let mut query = sqlx::query(sql);
  for arg in args {
    trace!(?arg, "binding arg");
    query = match arg {
      ConvertedType::I8(v) => query.bind(v),
      ConvertedType::I16(v) => query.bind(v),
      ConvertedType::I32(v) => query.bind(v),
      ConvertedType::I64(v) => query.bind(v),
      ConvertedType::U8(v) => query.bind(v.map(|v| v as i16)),
      ConvertedType::U16(v) => query.bind(v),
      ConvertedType::U32(v) => query.bind(v),
      ConvertedType::U64(v) => query.bind(v),
      ConvertedType::F32(v) => query.bind(v),
      ConvertedType::F64(v) => query.bind(v),
      ConvertedType::Bool(v) => query.bind(v),
      ConvertedType::String(v) => query.bind(v),
      ConvertedType::Datetime(v) => query.bind(v),
    };
  }
  query
}
