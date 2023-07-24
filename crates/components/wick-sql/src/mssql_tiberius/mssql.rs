use std::time::Duration;

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{AuthMethod, Config};
use url::Url;
use wick_config::config::components::SqlComponentConfig;

use crate::Error;

pub(crate) async fn connect(_config: &SqlComponentConfig, addr: &Url) -> Result<Pool<ConnectionManager>, Error> {
  debug!(connect = %addr, "connecting to mssql");
  let mut config = Config::new();

  let db = addr.path().trim_start_matches('/');
  config.database(db);
  if let Some(host) = addr.host() {
    config.host(host.to_string());
  }
  if let Some(port) = addr.port() {
    config.port(port);
  }
  if let (user, Some(password)) = (addr.username(), addr.password()) {
    config.authentication(AuthMethod::sql_server(user, password));
  }
  config.trust_cert();

  let mgr = bb8_tiberius::ConnectionManager::new(config);

  let pool = bb8::Pool::builder()
    .max_size(50)
    .connection_timeout(Duration::from_secs(30))
    .build(mgr)
    .await
    .map_err(|e| Error::Pool(e.to_string()))?;

  // Need to try and get a connection to ensure the pool is actually connected.
  let conn = pool.get().await.map_err(|e| Error::Pool(e.to_string()))?;
  drop(conn);

  Ok(pool)
}
