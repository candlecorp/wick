mod serialize;
mod wrapper;
pub(crate) use serialize::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use url::Url;
use wick_config::config::components::SqlComponentConfig;

use crate::Error;

pub(crate) async fn connect(_config: &SqlComponentConfig, addr: &Url) -> Result<PgPool, Error> {
  debug!(%addr, "connecting to postgres");

  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(addr.as_ref())
    .await
    .map_err(|e| Error::PostgresConnect(e.to_string()))?;
  Ok(pool)
}
