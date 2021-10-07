use std::collections::HashMap;
use std::sync::Arc;

use redis::aio::Connection;
use redis::FromRedisValue;
use tokio::sync::RwLock;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use crate::error::Error;
use crate::generated::{
  self,
  Dispatcher,
};

pub(crate) type Context = Arc<RedisConnection>;

#[allow(missing_debug_implementations)]
pub struct RedisConnection(RwLock<Connection>);

pub type RedisResult<T> = std::result::Result<T, Error>;

impl From<Error> for NativeComponentError {
  fn from(e: Error) -> Self {
    NativeComponentError::new(e.to_string())
  }
}

impl RedisConnection {
  pub async fn run_cmd<T: FromRedisValue + std::fmt::Debug>(
    &self,
    cmd: &mut redis::Cmd,
  ) -> RedisResult<T> {
    let mut con = self.0.write().await;
    let result: Result<T> = cmd
      .query_async(&mut *con)
      .await
      .map_err(|e| Error::RedisError(e.to_string()));

    if log_enabled!(log::Level::Trace) {
      let bytes = cmd.get_packed_command();
      let cmdstring = String::from_utf8_lossy(&bytes);
      trace!("REDIS:EXEC[{}]", cmdstring);

      trace!("REDIS:RESULT[{:?}]", result);
    }
    result
  }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
#[allow(missing_debug_implementations)]
pub struct State {
  pub connections: RwLock<HashMap<String, Context>>,
}

#[derive(Clone, Default)]
#[must_use]
#[allow(missing_debug_implementations)]
pub struct Provider {
  context: Arc<RwLock<State>>,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Arc::new(RwLock::new(State::default())),
    }
  }
  pub fn new() -> Self {
    Self::default()
  }
  pub async fn connect(&self, namespace: String, url: String) -> Result<()> {
    let client = redis::Client::open(url.clone())
      .map_err(|e| Error::Init(format!("connection to {}: {}", url, e)))?;

    let connection = client
      .get_async_connection()
      .await
      .map_err(|e| Error::Init(format!("connection to {}: {}", url, e)))?;

    let context = self.context.write().await;

    let mut update_map = context.connections.write().await;
    update_map.insert(
      namespace,
      Arc::new(RedisConnection(RwLock::new(connection))),
    );
    Ok(())
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.read().await;
    let connections = context.connections.read().await;
    let namespace = "default".to_owned();
    let connection = connections
      .get(&namespace)
      .ok_or_else(|| RpcError::ProviderError(Error::ConnectionNotFound(namespace).to_string()))?;
    debug!("Dispatching to {}", entity.url());
    let component = entity.name();
    let stream = Dispatcher::dispatch(&component, connection.clone(), payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = generated::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use rand::Rng;
  use vino_interface_keyvalue::generated::*;

  use super::*;

  async fn key_set(provider: &Provider, key: &str, value: &str, expires: u32) -> Result<bool> {
    debug!("key-set:{}::{}::{}", key, value, expires);
    let payload = key_set::Inputs {
      key: key.to_owned(),
      value: value.to_owned(),
      expires,
    };

    let mut outputs: key_set::Outputs = provider
      .invoke(Entity::component_direct("key-set"), payload.into())
      .await?
      .into();
    let actual = outputs.result().await?.try_next_into()?;

    Ok(actual)
  }

  async fn key_get(provider: &Provider, key: &str) -> Result<String> {
    debug!("key-get:{}", key);
    let payload = key_get::Inputs {
      key: key.to_owned(),
    };

    let mut outputs: key_get::Outputs = provider
      .invoke(Entity::component_direct("key-get"), payload.into())
      .await?
      .into();
    let actual = outputs.value().await?.try_next_into()?;

    Ok(actual)
  }

  async fn delete(provider: &Provider, key: &str) -> Result<u32> {
    debug!("delete:{}", key);
    let payload = delete::Inputs {
      keys: vec![key.to_owned()],
    };

    let mut outputs: delete::Outputs = provider
      .invoke(Entity::component_direct("delete"), payload.into())
      .await?
      .into();
    let actual = outputs.num().await?.try_next_into()?;
    Ok(actual)
  }

  async fn exists(provider: &Provider, key: &str) -> Result<bool> {
    debug!("exists:{}", key);
    let payload = exists::Inputs {
      key: key.to_owned(),
    };

    let mut outputs: exists::Outputs = provider
      .invoke(Entity::component_direct("exists"), payload.into())
      .await?
      .into();

    let actual = outputs.exists().await?.try_next_into()?;

    Ok(actual)
  }

  async fn list_add(provider: &Provider, key: &str, value: &str) -> Result<u32> {
    debug!("list-add:{}::{}", key, value);
    let payload = list_add::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };

    let mut outputs: list_add::Outputs = provider
      .invoke(Entity::component_direct("list-add"), payload.into())
      .await?
      .into();
    let actual = outputs.length().await?.try_next_into()?;

    Ok(actual)
  }

  async fn list_range(provider: &Provider, key: &str, start: i32, end: i32) -> Result<Vec<String>> {
    debug!("list-range:{}::{}::{}", key, start, end);
    let payload = list_range::Inputs {
      key: key.to_owned(),
      start,
      end,
    };

    let mut outputs: list_range::Outputs = provider
      .invoke(Entity::component_direct("list-range"), payload.into())
      .await?
      .into();
    let actual = outputs.values().await?.try_next_into()?;

    Ok(actual)
  }

  async fn list_remove(provider: &Provider, key: &str, value: &str) -> Result<u32> {
    debug!("list-remove:{}::{}", key, value);
    let payload = list_remove::Inputs {
      key: key.to_owned(),
      num: 1,
      value: value.to_owned(),
    };

    let mut outputs: list_remove::Outputs = provider
      .invoke(Entity::component_direct("list-remove"), payload.into())
      .await?
      .into();
    let actual = outputs.num().await?.try_next_into()?;

    Ok(actual)
  }

  async fn set_add(provider: &Provider, key: &str, value: &str) -> Result<u32> {
    debug!("set-add:{}::{}", key, value);
    let payload = set_add::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };

    let mut outputs: set_add::Outputs = provider
      .invoke(Entity::component_direct("set-add"), payload.into())
      .await?
      .into();

    let actual = outputs.length().await?.try_next_into()?;

    Ok(actual)
  }

  async fn set_get(provider: &Provider, key: &str) -> Result<Vec<String>> {
    debug!("set-get:{}", key);
    let payload = set_get::Inputs {
      key: key.to_owned(),
    };

    let mut outputs: set_get::Outputs = provider
      .invoke(Entity::component_direct("set-get"), payload.into())
      .await?
      .into();

    let actual = outputs.values().await?.try_next_into()?;

    Ok(actual)
  }

  async fn set_scan(
    provider: &Provider,
    key: &str,
    cursor: &str,
    count: u32,
  ) -> Result<(String, Vec<String>)> {
    debug!("set-scan:{}", key);
    let payload = set_scan::Inputs {
      key: key.to_owned(),
      cursor: cursor.to_owned(),
      count,
    };

    let mut outputs: set_scan::Outputs = provider
      .invoke(Entity::component_direct("set-scan"), payload.into())
      .await?
      .into();

    let values = outputs.values().await?.try_next_into()?;
    let cursor = outputs.cursor().await?.try_next_into()?;

    Ok((cursor, values))
  }

  async fn set_remove(provider: &Provider, key: &str, value: &str) -> Result<u32> {
    debug!("set-remove:{}::{}", key, value);
    let payload = set_remove::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };

    let mut outputs: set_remove::Outputs = provider
      .invoke(Entity::component_direct("set-remove"), payload.into())
      .await?
      .into();

    let actual = outputs.num().await?.try_next_into()?;

    Ok(actual)
  }

  async fn get_default_provider() -> Result<Provider> {
    let provider = Provider::default();
    let url = std::env::var(crate::REDIS_URL_ENV.to_owned())
      .unwrap_or_else(|_| "redis://0.0.0.0:6379".to_owned());
    provider.connect("default".to_owned(), url).await?;
    Ok(provider)
  }

  fn get_random_string() -> String {
    rand::thread_rng()
      .sample_iter(&rand::distributions::Alphanumeric)
      .take(30)
      .map(char::from)
      .collect()
  }

  #[test_logger::test(tokio::test)]
  async fn test_key_set_get_contains_delete() -> Result<()> {
    let provider = get_default_provider().await?;
    let nonexistant_key = get_random_string();
    let key = get_random_string();
    let expected = get_random_string();
    let expires = 10000;

    assert!(!exists(&provider, &key).await?);
    let result = key_set(&provider, &key, &expected, expires).await?;
    assert!(result);
    let actual = key_get(&provider, &key).await?;
    assert_eq!(actual, expected);
    let result = key_get(&provider, &nonexistant_key).await;
    assert!(result.is_err());
    delete(&provider, &key).await?;
    assert!(!exists(&provider, &key).await?);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> Result<()> {
    let provider = get_default_provider().await?;
    let key = get_random_string();
    let expected = get_random_string();

    assert!(!exists(&provider, &key).await?);
    let _num = list_add(&provider, &key, &expected).await?;
    assert!(exists(&provider, &key).await?);
    let values = list_range(&provider, &key, 0, 1).await?;
    let range = vec![expected.clone()];
    assert_eq!(values, range);
    let mut rest = vec![
      get_random_string(),
      get_random_string(),
      get_random_string(),
      get_random_string(),
    ];
    list_add(&provider, &key, &rest[0]).await?;
    list_add(&provider, &key, &rest[1]).await?;
    list_add(&provider, &key, &rest[2]).await?;
    list_add(&provider, &key, &rest[3]).await?;
    let values = list_range(&provider, &key, 0, 0).await?;
    assert_eq!(values, range);
    let values = list_range(&provider, &key, 0, 1).await?;
    assert_eq!(values, vec![expected.clone(), rest[0].clone()]);
    let values = list_range(&provider, &key, 0, -1).await?;
    let mut all = range.clone();
    all.append(&mut rest);
    assert_eq!(values, all);
    list_remove(&provider, &key, &expected).await?;
    let values = list_range(&provider, &key, 0, -1).await?;
    assert_eq!(values, &all[1..]);
    delete(&provider, &key).await?;
    let values = list_range(&provider, &key, 0, -1).await?;
    let none: Vec<String> = vec![];
    assert_eq!(values, none);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_set_add_get_remove() -> Result<()> {
    let provider = get_default_provider().await?;
    let key = get_random_string();
    let expected = get_random_string();
    let range = vec![expected.clone()];

    assert!(!exists(&provider, &key).await?);
    set_add(&provider, &key, &expected).await?;
    assert!(exists(&provider, &key).await?);
    let values = set_get(&provider, &key).await?;
    assert_eq!(values, range);
    set_add(&provider, &key, &expected).await?;
    let values = set_get(&provider, &key).await?;
    assert_eq!(values, range);
    set_remove(&provider, &key, &expected).await?;
    let values = set_get(&provider, &key).await?;
    let none: Vec<String> = vec![];
    assert_eq!(values, none);
    assert!(!exists(&provider, &key).await?);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_set_scan() -> Result<()> {
    let provider = get_default_provider().await?;
    let key = get_random_string();

    let m1 = get_random_string();
    let m2 = get_random_string();
    let m3 = get_random_string();
    let mut all = [m1.clone(), m2.clone(), m3.clone()];
    set_add(&provider, &key, &m1).await?;
    set_add(&provider, &key, &m2).await?;
    set_add(&provider, &key, &m3).await?;
    let (cursor, values) = set_scan(&provider, &key, "0", 1).await?;
    assert!(values.len() >= 1);
    assert!(all.contains(&values[0]));
    let (cursor, values) = set_scan(&provider, &key, &cursor, 1).await?;
    assert!(values.len() >= 1);
    assert!(all.contains(&values[0]));

    delete(&provider, &key).await?;
    Ok(())
  }
}
