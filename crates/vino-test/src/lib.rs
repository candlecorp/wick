use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use serde::{
  Deserialize,
  Serialize,
};
use tap::{
  TestBlock,
  TestRunner,
};
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_rpc::BoxedRpcHandler;
use vino_transport::{
  Failure,
  MessageTransport,
  Success,
  TransportMap,
};

use self::error::TestError;

pub mod error;
pub use error::TestError as Error;

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
  pub error_kind: Option<String>,
  pub error_msg: Option<String>,
}

pub fn read_data(path: PathBuf) -> Result<Vec<TestData>, error::TestError> {
  let contents = read_to_string(path).map_err(|e| TestError::ReadFailed(e.to_string()))?;
  let data: Vec<TestData> =
    serde_yaml::from_str(&contents).map_err(|e| TestError::ParseFailed(e.to_string()))?;
  Ok(data)
}

pub async fn run_test(
  name: String,
  expected: Vec<TestData>,
  provider: BoxedRpcHandler,
) -> Result<TestRunner, Error> {
  let mut harness = TestRunner::new(Some(name));

  for test in expected {
    let mut payload = TransportMap::new();
    let entity = Entity::component_direct(test.component.clone());
    for (k, v) in &test.inputs {
      payload.insert(k, MessageTransport::Success(Success::Serialized(v.clone())));
    }

    let stream = provider
      .invoke(entity, payload)
      .await
      .map_err(|e| Error::InvocationFailed(e.to_string()))?;

    let outputs: Vec<_> = stream.collect().await;
    let description = test
      .description
      .map_or_else(String::new, |desc| format!(" - {}", desc));
    let mut test_block = TestBlock::new(Some(format!(
      "Component '{}'{}",
      test.component, description
    )));

    for (i, output) in test.outputs.into_iter().enumerate() {
      let result = outputs[i].port == output.port;
      let diag = Some(vec![
        format!("Actual: {}", outputs[i].port),
        format!("Expected: {}", output.port),
      ]);
      test_block.add_test(
        move || result,
        format!("Output port name is '{}'", output.port),
        diag,
      );

      if let Some(value) = &output.payload.value {
        let actual_payload = outputs[i].payload.clone();

        let actual_value: Result<serde_value::Value, Error> = actual_payload
          .try_into()
          .map_err(|e| Error::ConversionFailed(e.to_string()));
        let expected_value = value.clone();

        let diagnostic = Some(vec![
          format!(
            "Actual: {:?}",
            match &actual_value {
              Ok(v) => format!("{:?}", v),
              Err(e) => format!("Could not deserialize payload, message was : {}", e),
            }
          ),
          format!("Expected: {:?}", expected_value),
        ]);

        test_block.add_test(
          move || match actual_value {
            Ok(val) => val == expected_value,
            Err(_e) => false,
          },
          "Payload value matches",
          diagnostic,
        );
      }
      if let Some(error_kind) = output.payload.error_kind {
        let actual_payload = outputs[i].payload.clone();

        let diag = Some(vec![format!(
          "Expected an {} error kind, but payload was: {:?}",
          error_kind, actual_payload
        )]);

        test_block.add_test(
          move || match actual_payload {
            MessageTransport::Failure(Failure::Exception(_)) => (error_kind == "Exception"),
            MessageTransport::Failure(Failure::Error(_)) => (error_kind == "Error"),
            _ => false,
          },
          "Error kind matches",
          diag,
        );
      }
      if let Some(error_msg) = output.payload.error_msg {
        let actual_payload = outputs[i].payload.clone();

        let diag = Some(vec![format!(
          "Expected error message '{}', but payload was: {:?}",
          error_msg, actual_payload
        )]);

        test_block.add_test(
          move || match actual_payload {
            MessageTransport::Failure(Failure::Exception(msg)) => (error_msg == msg),
            MessageTransport::Failure(Failure::Error(msg)) => (error_msg == msg),
            _ => false,
          },
          "Error message matches",
          diag,
        );
      }
    }
    harness.add_block(test_block);
  }
  Ok(harness)
}
