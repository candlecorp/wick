extern crate rmp_serde as rmps;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize, Debug)]
pub enum Signal {
  Done,
  Waiting,
  HostError,
}

impl Default for Signal {
  fn default() -> Self {
    Signal::Done
  }
}

impl Display for Signal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Signal::Done => "done".to_string(),
        Signal::Waiting => "waiting".to_string(),
        Signal::HostError => "host error".to_string(),
      },
    )
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputPayload {
  MessagePack(Vec<u8>),
  Exception(String),
  Error(String),
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct ConnectionDownstream {
  pub namespace: String,
  pub host_id: String,
  pub tx_id: String,
  pub actor: String,
  pub reference: String,
}
