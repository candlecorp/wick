use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use redis::aio::Connection;
use redis::{FromRedisValue, Pipeline};
use tokio::sync::RwLock;
use tracing::Instrument;
use wasmflow_rpc::error::RpcError;
use wasmflow_rpc::{RpcHandler, RpcResult};
use wasmflow_sdk::v1::stateful::NativeDispatcher;
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::types::HostedType;
use wasmflow_sdk::v1::Invocation;

use crate::components::ComponentDispatcher;
use crate::error::Error;

pub(crate) type Context = Arc<RedisConnection>;

pub struct RedisConnection(RwLock<Connection>);

impl std::fmt::Debug for RedisConnection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("RedisConnection()").finish()
  }
}

pub type RedisResult<T> = std::result::Result<T, Error>;

impl RedisConnection {
  pub async fn run_cmd<T: FromRedisValue + std::fmt::Debug>(&self, cmd: &mut redis::Cmd) -> RedisResult<T> {
    let mut con = self.0.write().await;
    let now = Instant::now();
    let result: Result<T> = cmd
      .query_async(&mut *con)
      .instrument(trace_span!("redis query exec"))
      .await
      .map_err(|e| Error::RedisError(e.to_string()));
    trace!(duration_ns = %now.elapsed().as_micros(), "redis query exec complete",);

    result
  }

  pub async fn run_pipeline<T: FromRedisValue + std::fmt::Debug>(&self, pipeline: &mut Pipeline) -> RedisResult<T> {
    let mut con = self.0.write().await;
    let now = Instant::now();

    let result = pipeline
      .query_async(&mut *con)
      .instrument(trace_span!("redis pipeline exec"))
      .await
      .map_err(|e| Error::RedisError(e.to_string()));

    trace!(duration_ns = %now.elapsed().as_micros(), "redis pipeline exec complete",);

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
pub struct Collection {
  context: Arc<RwLock<State>>,
}

impl Collection {
  pub fn new() -> Self {
    Self::default()
  }
  pub async fn connect(&self, namespace: String, url: String) -> Result<()> {
    let client = redis::Client::open(url.clone()).map_err(|e| Error::Init(format!("connection to {}: {}", url, e)))?;

    let connection = client
      .get_async_connection()
      .await
      .map_err(|e| Error::Init(format!("connection to {}: {}", url, e)))?;

    let context = self.context.write().await;

    let mut update_map = context.connections.write().await;
    update_map.insert(namespace, Arc::new(RedisConnection(RwLock::new(connection))));
    Ok(())
  }
}

#[async_trait::async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation) -> RpcResult<TransportStream> {
    let context = self.context.read().await;
    let connections = context.connections.read().await;
    let namespace = "default".to_owned();
    let connection = connections
      .get(&namespace)
      .ok_or_else(|| RpcError::CollectionError(Error::ConnectionNotFound(namespace).to_string()))?;
    let dispatcher = ComponentDispatcher::default();
    let stream = dispatcher
      .dispatch(invocation, connection.clone())
      .await
      .map_err(|e| RpcError::CollectionError(e.to_string()))?;

    Ok(TransportStream::from_packetstream(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = crate::components::get_signature();
    Ok(vec![HostedType::Collection(signature)])
  }
}

#[cfg(test)]
mod integration {

  use anyhow::Result;
  use rand::Rng;
  use wasmflow_interface_keyvalue::*;
  use wasmflow_sdk::v1::Entity;

  use super::*;
  use crate::components::generated::__batch__::{self, ComponentInputs};

  async fn key_set(collection: &Collection, key: &str, value: &str, expires: u32) -> Result<bool> {
    debug!("key-set:{}::{}::{}", key, value, expires);
    let payload = key_set::Inputs {
      key: key.to_owned(),
      value: value.to_owned(),
      expires,
    };

    let invocation = Invocation::new_test(file!(), Entity::local("key-set"), payload, None);

    let mut outputs: key_set::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.result().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn key_get(collection: &Collection, key: &str) -> Result<String> {
    debug!("key-get:{}", key);
    let payload = key_get::Inputs { key: key.to_owned() };
    let invocation = Invocation::new_test(file!(), Entity::local("key-get"), payload, None);

    let mut outputs: key_get::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.value().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn delete(collection: &Collection, key: &str) -> Result<u32> {
    debug!("delete:{}", key);
    let payload = delete::Inputs {
      keys: vec![key.to_owned()],
    };
    let invocation = Invocation::new_test(file!(), Entity::local("delete"), payload, None);

    let mut outputs: delete::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.num().await?.deserialize_next()?;
    Ok(actual)
  }

  async fn exists(collection: &Collection, key: &str) -> Result<bool> {
    debug!("exists:{}", key);
    let payload = exists::Inputs { key: key.to_owned() };
    let invocation = Invocation::new_test(file!(), Entity::local("exists"), payload, None);

    let mut outputs: exists::Outputs = collection.invoke(invocation).await?.into();

    let actual = outputs.exists().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn list_add(collection: &Collection, key: &str, value: &str) -> Result<u32> {
    debug!("list-add:{}::{}", key, value);
    let payload = list_add::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };
    let invocation = Invocation::new_test(file!(), Entity::local("list-add"), payload, None);

    let mut outputs: list_add::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.length().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn list_range(collection: &Collection, key: &str, start: i32, end: i32) -> Result<Vec<String>> {
    debug!("list-range:{}::{}::{}", key, start, end);
    let payload = list_range::Inputs {
      key: key.to_owned(),
      start,
      end,
    };
    let invocation = Invocation::new_test(file!(), Entity::local("list-range"), payload, None);

    let mut outputs: list_range::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.values().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn list_remove(collection: &Collection, key: &str, value: &str) -> Result<u32> {
    debug!("list-remove:{}::{}", key, value);
    let payload = list_remove::Inputs {
      key: key.to_owned(),
      num: 1,
      value: value.to_owned(),
    };
    let invocation = Invocation::new_test(file!(), Entity::local("list-remove"), payload, None);

    let mut outputs: list_remove::Outputs = collection.invoke(invocation).await?.into();
    let actual = outputs.num().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn set_add(collection: &Collection, key: &str, value: &str) -> Result<u32> {
    debug!("set-add:{}::{}", key, value);
    let payload = set_add::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };
    let invocation = Invocation::new_test(file!(), Entity::local("set-add"), payload, None);

    let mut outputs: set_add::Outputs = collection.invoke(invocation).await?.into();

    let actual = outputs.length().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn set_get(collection: &Collection, key: &str) -> Result<Vec<String>> {
    debug!("set-get:{}", key);
    let payload = set_get::Inputs { key: key.to_owned() };
    let invocation = Invocation::new_test(file!(), Entity::local("set-get"), payload, None);

    let mut outputs: set_get::Outputs = collection.invoke(invocation).await?.into();

    let actual = outputs.values().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn set_scan(collection: &Collection, key: &str, cursor: &str, count: u32) -> Result<(String, Vec<String>)> {
    debug!("set-scan:{}", key);
    let payload = set_scan::Inputs {
      key: key.to_owned(),
      cursor: cursor.to_owned(),
      count,
    };
    let invocation = Invocation::new_test(file!(), Entity::local("set-scan"), payload, None);

    let mut outputs: set_scan::Outputs = collection.invoke(invocation).await?.into();

    let values = outputs.values().await?.deserialize_next()?;
    let cursor = outputs.cursor().await?.deserialize_next()?;

    Ok((cursor, values))
  }

  async fn set_remove(collection: &Collection, key: &str, value: &str) -> Result<u32> {
    debug!("set-remove:{}::{}", key, value);
    let payload = set_remove::Inputs {
      key: key.to_owned(),
      values: vec![value.to_owned()],
    };
    let invocation = Invocation::new_test(file!(), Entity::local("set-remove"), payload, None);

    let mut outputs: set_remove::Outputs = collection.invoke(invocation).await?.into();

    let actual = outputs.num().await?.deserialize_next()?;

    Ok(actual)
  }

  async fn get_default_collection() -> Result<Collection> {
    let collection = Collection::default();
    let url = std::env::var(crate::REDIS_URL_ENV).unwrap_or_else(|_| "redis://0.0.0.0:6379".to_owned());
    collection.connect("default".to_owned(), url).await?;
    Ok(collection)
  }

  fn get_random_string() -> String {
    rand::thread_rng()
      .sample_iter(&rand::distributions::Alphanumeric)
      .take(30)
      .map(char::from)
      .collect()
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_key_set_get_contains_delete() -> Result<()> {
    let collection = get_default_collection().await?;
    let nonexistant_key = get_random_string();
    let key = get_random_string();
    let expected = get_random_string();
    let expires = 10000;

    assert!(!exists(&collection, &key).await?);
    let result = key_set(&collection, &key, &expected, expires).await?;
    assert!(result);
    let actual = key_get(&collection, &key).await?;
    assert_eq!(actual, expected);
    let result = key_get(&collection, &nonexistant_key).await;
    assert!(result.is_err());
    delete(&collection, &key).await?;
    assert!(!exists(&collection, &key).await?);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_list() -> Result<()> {
    let collection = get_default_collection().await?;
    let key = get_random_string();
    let expected = get_random_string();

    assert!(!exists(&collection, &key).await?);
    let _num = list_add(&collection, &key, &expected).await?;
    assert!(exists(&collection, &key).await?);
    let values = list_range(&collection, &key, 0, 1).await?;
    let range = vec![expected.clone()];
    assert_eq!(values, range);
    let mut rest = vec![
      get_random_string(),
      get_random_string(),
      get_random_string(),
      get_random_string(),
    ];
    list_add(&collection, &key, &rest[0]).await?;
    list_add(&collection, &key, &rest[1]).await?;
    list_add(&collection, &key, &rest[2]).await?;
    list_add(&collection, &key, &rest[3]).await?;
    let values = list_range(&collection, &key, 0, 0).await?;
    assert_eq!(values, range);
    let values = list_range(&collection, &key, 0, 1).await?;
    assert_eq!(values, vec![expected.clone(), rest[0].clone()]);
    let values = list_range(&collection, &key, 0, -1).await?;
    let mut all = range.clone();
    all.append(&mut rest);
    assert_eq!(values, all);
    list_remove(&collection, &key, &expected).await?;
    let values = list_range(&collection, &key, 0, -1).await?;
    assert_eq!(values, &all[1..]);
    delete(&collection, &key).await?;
    let values = list_range(&collection, &key, 0, -1).await?;
    let none: Vec<String> = vec![];
    assert_eq!(values, none);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_set_add_get_remove() -> Result<()> {
    let collection = get_default_collection().await?;
    let key = get_random_string();
    let expected = get_random_string();
    let range = vec![expected.clone()];

    assert!(!exists(&collection, &key).await?);
    set_add(&collection, &key, &expected).await?;
    assert!(exists(&collection, &key).await?);
    let values = set_get(&collection, &key).await?;
    assert_eq!(values, range);
    set_add(&collection, &key, &expected).await?;
    let values = set_get(&collection, &key).await?;
    assert_eq!(values, range);
    set_remove(&collection, &key, &expected).await?;
    let values = set_get(&collection, &key).await?;
    let none: Vec<String> = vec![];
    assert_eq!(values, none);
    assert!(!exists(&collection, &key).await?);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_set_scan() -> Result<()> {
    let collection = get_default_collection().await?;
    let key = get_random_string();

    let m1 = get_random_string();
    let m2 = get_random_string();
    let m3 = get_random_string();
    let all = [m1.clone(), m2.clone(), m3.clone()];
    set_add(&collection, &key, &m1).await?;
    set_add(&collection, &key, &m2).await?;
    set_add(&collection, &key, &m3).await?;
    let (cursor, values) = set_scan(&collection, &key, "0", 1).await?;
    println!("first values: {:?}", values);
    assert!(!values.is_empty());
    assert!(all.contains(&values[0]));
    let (_cursor, values) = set_scan(&collection, &key, &cursor, 1).await?;
    println!("next values: {:?}", values);
    assert!(!values.is_empty());
    assert!(all.contains(&values[0]));

    delete(&collection, &key).await?;
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_multi() -> Result<()> {
    let collection = get_default_collection().await?;
    let key = get_random_string();
    let list_key = get_random_string();

    let uuid = "MY_UUID".to_owned();

    let key_set_payload = ComponentInputs::KeySet(key_set::Inputs {
      key: key.clone(),
      value: uuid.clone(),
      expires: 0,
    });
    let list_add_payload = ComponentInputs::ListAdd(list_add::Inputs {
      key: list_key.clone(),
      values: vec![uuid.clone()],
    });
    let payloads = vec![key_set_payload, list_add_payload];
    let inputs = __batch__::Inputs { inputs: payloads };

    let invocation = Invocation::new_test(file!(), Entity::local("__batch__"), inputs, None);

    let mut outputs: __batch__::Outputs = collection.invoke(invocation).await?.into();

    let result: bool = outputs.result().await?.deserialize_next()?;
    assert!(result);

    let value = key_get(&collection, &key).await?;
    assert_eq!(value, uuid);
    let values = list_range(&collection, &list_key, 0, -1).await?;
    assert_eq!(values, vec![uuid]);

    delete(&collection, &key).await?;
    delete(&collection, &list_key).await?;
    Ok(())
  }
}
