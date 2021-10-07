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
  TransportWrapper,
};

use self::error::TestError;

pub mod error;
pub use error::TestError as Error;

#[macro_use]
extern crate log;

pub struct TestSuite {
  tests: Vec<TestData>,
  name: String,
  filters: Vec<String>,
}

impl Default for TestSuite {
  fn default() -> Self {
    Self::new("Test")
  }
}

impl TestSuite {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      tests: Vec::new(),
      name: name.as_ref().to_owned(),
      filters: Vec::new(),
    }
  }

  pub fn try_from_file(file: PathBuf) -> Result<Self, TestError> {
    let contents = read_to_string(file).map_err(|e| TestError::ReadFailed(e.to_string()))?;
    let data: Vec<TestData> =
      serde_yaml::from_str(&contents).map_err(|e| TestError::ParseFailed(e.to_string()))?;

    Ok(TestSuite::from_tests(data))
  }

  pub fn from_tests(tests: Vec<TestData>) -> Self {
    Self {
      tests,
      ..Default::default()
    }
  }

  pub fn get_tests(&mut self) -> Vec<&mut TestData> {
    let filters = &self.filters;
    if !filters.is_empty() {
      self
        .tests
        .iter_mut()
        .filter(|test| {
          filters
            .iter()
            .any(|filter| test.get_description().contains(filter))
        })
        .collect()
    } else {
      self.tests.iter_mut().collect()
    }
  }

  pub fn filter(mut self, filters: Vec<String>) -> Self {
    self.filters = filters;
    self
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub async fn run(&mut self, provider: BoxedRpcHandler) -> Result<TestRunner, TestError> {
    let name = self.name.clone();
    let tests = self.get_tests();
    run_test(name, tests, provider).await
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestData {
  pub component: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub init_data: HashMap<String, String>,
  #[serde(default)]
  pub inputs: HashMap<String, serde_value::Value>,
  #[serde(default)]
  pub outputs: Vec<OutputData>,
  #[serde(skip)]
  pub actual: Vec<TransportWrapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct OutputData {
  pub port: String,
  pub payload: SerializedTransport,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SerializedTransport {
  pub value: Option<serde_value::Value>,
  pub error_kind: Option<String>,
  pub error_msg: Option<String>,
}

impl TestData {
  pub fn get_payload(&self) -> TransportMap {
    let mut payload = TransportMap::new();
    for (k, v) in &self.inputs {
      debug!("Test input for port '{}': {:?}", k, v);
      payload.insert(k, MessageTransport::Success(Success::Serialized(v.clone())));
    }

    if !self.init_data.is_empty() {
      payload.with_config(self.init_data.clone());
    }
    payload
  }

  pub fn get_description(&self) -> String {
    let separator = if self.description.is_empty() {
      ""
    } else {
      " : "
    };
    format!("{}{}{}", self.component, separator, self.description)
  }
}

pub async fn run_test(
  name: String,
  expected: Vec<&mut TestData>,
  provider: BoxedRpcHandler,
) -> Result<TestRunner, Error> {
  let mut harness = TestRunner::new(Some(name));

  for (i, test) in expected.into_iter().enumerate() {
    let payload = test.get_payload();
    let entity = Entity::component_direct(test.component.clone());

    let mut test_block = TestBlock::new(Some(test.get_description()));

    trace!("TEST[{}]:INVOKE[{}]", i, entity,);
    trace!("TEST[{}]:PAYLOAD:{:?}", i, payload,);
    let result = provider
      .invoke(entity, payload)
      .await
      .map_err(|e| Error::InvocationFailed(e.to_string()));

    if let Err(e) = result {
      test_block.add_test(
        || false,
        "Invocation",
        Some(vec![format!("Invocation failed: {}", e)]),
      );
      harness.add_block(test_block);
      continue;
    }

    let stream = result.unwrap();

    let outputs: Vec<_> = stream.collect().await;
    test.actual = outputs;
    let mut diagnostics = vec!["Output: ".to_owned()];
    let mut output_lines: Vec<_> = test.actual.iter().map(|o| format!("{:?}", o)).collect();
    diagnostics.append(&mut output_lines);
    test_block.diagnostics = diagnostics;

    for (j, expected) in test.outputs.iter().cloned().enumerate() {
      if j >= test.actual.len() {
        test_block.add_test(
          || false,
          "Stream length",
          Some(vec![
            "Test data included more output than component produced".to_owned(),
          ]),
        );
        break;
      }
      let actual = &test.actual[j];
      let result = actual.port == expected.port;
      let diag = Some(vec![
        format!("Actual: {}", actual.port),
        format!("Expected: {}", expected.port),
      ]);
      test_block.add_test(
        move || result,
        format!("Output port name == '{}'", expected.port),
        diag,
      );

      if let Some(value) = &expected.payload.value {
        let actual_payload = actual.payload.clone();

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

        debug!("TEST[{}:{}]:ACTUAL:{:?}", i, j, actual_value);
        debug!("TEST[{}:{}]:EXPECTED:{:?}", i, j, expected_value);
        test_block.add_test(
          move || match actual_value {
            Ok(val) => eq(val, expected_value),
            Err(_e) => false,
          },
          "Payload value",
          diagnostic,
        );
      }
      if let Some(error_kind) = expected.payload.error_kind {
        let actual_payload = actual.payload.clone();

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
          "Error kind",
          diag,
        );
      }
      if let Some(error_msg) = expected.payload.error_msg {
        let actual_payload = actual.payload.clone();

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
          "Error message",
          diag,
        );
      }
    }
    let num_tested = test.outputs.len();
    let mut missed = vec![];
    for i in num_tested..test.actual.len() {
      if let Some(output) = test.actual.get(i) {
        if !matches!(output.payload, MessageTransport::Signal(_)) {
          debug!("TEST:MISSED:{:?}", output);
          missed.push(output);
        }
      }
    }
    let num_missed = missed.len();
    test_block.add_test(
      move || num_missed == 0,
      "Tested all outputs",
      Some(missed.into_iter().map(|p| format!("{:?}", p)).collect()),
    );

    harness.add_block(test_block);
  }
  harness.run();
  Ok(harness)
}

fn eq(left: serde_value::Value, right: serde_value::Value) -> bool {
  promote_val(left) == promote_val(right)
}

fn promote_val(val: serde_value::Value) -> serde_value::Value {
  use serde_value::Value;
  match val {
    Value::U8(n) => Value::U64(n.into()),
    Value::U16(n) => Value::U64(n.into()),
    Value::U32(n) => Value::U64(n.into()),
    Value::I8(n) => Value::I64(n.into()),
    Value::I16(n) => Value::I64(n.into()),
    Value::I32(n) => Value::I64(n.into()),
    Value::F32(n) => Value::F64(n.into()),
    Value::Char(n) => Value::String(n.into()),
    x => x,
  }
}
