mod serialize;
mod wrapper;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use url::Url;
use wick_config::config::components::SqlComponentConfig;

pub(crate) use self::serialize::*;
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
