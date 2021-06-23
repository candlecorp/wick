pub mod error;
pub mod generated;
pub mod invocation_server;
pub mod port;
use std::collections::HashMap;

use async_trait::async_trait;
pub use generated::vino as rpc;
pub use invocation_server::InvocationServer;
use serde::{
  Deserialize,
  Serialize,
};

pub type Result<T> = std::result::Result<T, error::RpcError>;
pub type Error = crate::error::RpcError;
pub type RpcResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_use]
extern crate log;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Component {
  pub name: String,
  pub inputs: Vec<Port>,
  pub outputs: Vec<Port>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Port {
  pub name: String,
  pub type_string: String,
}

impl From<(String, String)> for Port {
  fn from(tup: (String, String)) -> Self {
    let (name, type_string) = tup;
    Self { name, type_string }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Provider {
  pub name: String,
  pub components: Vec<Component>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Schematic {
  pub name: String,
  pub inputs: Vec<Port>,
  pub outputs: Vec<Port>,
  pub provider: Vec<Provider>,
  pub components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HostedType {
  Component(Component),
  Provider(Provider),
  Schematic(Schematic),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Statistics {
  pub num_calls: usize,
  pub execution_duration: ExecutionStatistics,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExecutionStatistics {
  pub max_time: usize,
  pub min_time: usize,
  pub average: usize,
}

#[async_trait]
pub trait RpcHandler: Send + Sync {
  async fn request(
    &self,
    inv_id: String,
    component: String,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<crate::port::Receiver>;
  async fn list_registered(&self) -> RpcResult<Vec<crate::HostedType>>;
  async fn report_statistics(&self, id: Option<String>) -> RpcResult<Vec<crate::Statistics>>;
}
