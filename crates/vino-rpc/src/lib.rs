pub mod error;
pub mod generated;
pub mod invocation_server;
pub mod port;
use std::collections::HashMap;
use std::iter::FromIterator;

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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Component {
  pub name: String,
  pub inputs: Vec<Port>,
  pub outputs: Vec<Port>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Provider {
  pub name: String,
  pub components: Vec<Component>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Schematic {
  pub name: String,
  pub inputs: Vec<Port>,
  pub outputs: Vec<Port>,
  pub provider: Vec<Provider>,
  pub components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HostedType {
  Component(Component),
  Schematic(Schematic),
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
  pub num_calls: u64,
  pub execution_duration: ExecutionStatistics,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
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

impl FromIterator<HostedType> for Vec<crate::rpc::Component> {
  fn from_iter<T: IntoIterator<Item = HostedType>>(iter: T) -> Self {
    iter.into_iter().collect()
  }
}
impl FromIterator<crate::Port> for Vec<crate::rpc::component::Port> {
  fn from_iter<T: IntoIterator<Item = crate::Port>>(iter: T) -> Self {
    iter.into_iter().collect()
  }
}
impl FromIterator<crate::Statistics> for Vec<crate::rpc::Statistic> {
  fn from_iter<T: IntoIterator<Item = crate::Statistics>>(iter: T) -> Self {
    iter.into_iter().collect()
  }
}
impl From<HostedType> for crate::rpc::Component {
  fn from(v: HostedType) -> Self {
    match v {
      HostedType::Component(v) => v.into(),
      HostedType::Schematic(v) => v.into(),
    }
  }
}

impl From<crate::Component> for crate::generated::vino::Component {
  fn from(v: crate::Component) -> Self {
    Self {
      name: v.name,
      kind: crate::rpc::component::ComponentKind::Component.into(),
      inputs: v.inputs.into_iter().collect(),
      outputs: v.outputs.into_iter().collect(),
    }
  }
}

impl From<crate::Schematic> for crate::generated::vino::Component {
  fn from(v: crate::Schematic) -> Self {
    Self {
      name: v.name,
      kind: crate::rpc::component::ComponentKind::Schematic.into(),
      inputs: v.inputs.into_iter().collect(),
      outputs: v.outputs.into_iter().collect(),
    }
  }
}

impl From<crate::Port> for crate::generated::vino::component::Port {
  fn from(v: crate::Port) -> Self {
    Self {
      name: v.name,
      r#type: v.type_string,
    }
  }
}

impl From<crate::Statistics> for crate::generated::vino::Statistic {
  fn from(v: crate::Statistics) -> Self {
    Self {
      num_calls: v.num_calls,
    }
  }
}
