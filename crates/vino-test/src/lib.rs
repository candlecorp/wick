use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use serde::{
  Deserialize,
  Serialize,
};

use self::error::TestError;

pub mod error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestData {
  pub component: String,
  pub description: Option<String>,
  pub inputs: HashMap<String, serde_value::Value>,
  pub outputs: Vec<OutputData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputData {
  pub port: String,
  pub payload: SerializedTransport,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializedTransport {
  pub value: Option<serde_value::Value>,
  pub error_kind: Option<serde_value::Value>,
  pub error_msg: Option<serde_value::Value>,
}

pub fn read_data(path: PathBuf) -> Result<Vec<TestData>, error::TestError> {
  let contents = read_to_string(path).map_err(|e| TestError::ReadFailed(e.to_string()))?;
  let data: Vec<TestData> =
    serde_yaml::from_str(&contents).map_err(|e| TestError::ParseFailed(e.to_string()))?;
  Ok(data)
}
