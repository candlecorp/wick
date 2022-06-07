// !!START_LINTS
// Wasmflow lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tap::{TestBlock, TestRunner};
use tokio_stream::StreamExt;
use wasmflow_entity::Entity;
use wasmflow_invocation::{InherentData, Invocation};
use wasmflow_packet::PacketMap;
use wasmflow_rpc::SharedRpcHandler;
use wasmflow_transport::{Failure, MessageTransport, Serialized, TransportWrapper};

use self::error::TestError;

pub mod error;
pub use error::TestError as Error;

#[macro_use]
extern crate tracing;

#[derive(Debug)]
#[must_use]
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
    let data: Vec<TestData> = serde_yaml::from_str(&contents).map_err(|e| TestError::ParseFailed(e.to_string()))?;

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
        .filter(|test| filters.iter().any(|filter| test.get_description().contains(filter)))
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

  pub async fn run(&mut self, collection: SharedRpcHandler) -> Result<TestRunner, TestError> {
    let name = self.name.clone();
    let tests = self.get_tests();
    run_test(name, tests, collection).await
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestData {
  pub component: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub seed: Option<u64>,
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
  pub fn get_payload(&self) -> (PacketMap, Option<InherentData>) {
    let mut payload = PacketMap::default();
    for (k, v) in &self.inputs {
      debug!("Test input for port '{}': {:?}", k, v);
      payload.insert(k, MessageTransport::Success(Serialized::Struct(v.clone())));
    }

    if let Some(seed) = self.seed {
      (
        payload,
        Some(InherentData::new(
          seed,
          SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap(),
        )),
      )
    } else {
      (payload, None)
    }
  }

  #[must_use]
  pub fn get_description(&self) -> String {
    let separator = if self.description.is_empty() { "" } else { " : " };
    format!("{}{}{}", self.component, separator, self.description)
  }
}

#[allow(clippy::too_many_lines)]
#[instrument(skip_all, name = "test_run")]
pub async fn run_test(
  name: String,
  expected: Vec<&mut TestData>,
  collection: SharedRpcHandler,
) -> Result<TestRunner, Error> {
  let mut harness = TestRunner::new(Some(name));

  for (i, test) in expected.into_iter().enumerate() {
    let (payload, inherent) = test.get_payload();
    let entity = Entity::local(test.component.clone());
    let test_name = test.get_description();
    let mut test_block = TestBlock::new(Some(test_name.clone()));
    let prefix = |msg: &str| format!("{}:{}", test_name, msg);

    trace!(i, %entity, "invoke");
    trace!(i, ?payload, "payload");
    let invocation = Invocation::new_test(&test_name, entity, payload, inherent);
    let result = collection
      .invoke(invocation)
      .await
      .map_err(|e| Error::InvocationFailed(e.to_string()));

    if let Err(e) = result {
      test_block.add_test(
        || false,
        prefix("invocation"),
        Some(vec![format!("Invocation failed: {}", e)]),
      );
      harness.add_block(test_block);
      continue;
    }

    let stream = result.unwrap();

    let outputs: Vec<_> = stream
      .filter(|msg| !msg.payload.is_signal())
      .map(TransportWrapper::from)
      .collect()
      .await;
    test.actual = outputs;
    let mut diagnostics = vec!["Output: ".to_owned()];
    let mut output_lines: Vec<_> = test.actual.iter().map(|o| format!("{:?}", o)).collect();
    diagnostics.append(&mut output_lines);
    test_block.diagnostics = diagnostics;

    for (j, expected) in test.outputs.iter().cloned().enumerate() {
      if j >= test.actual.len() {
        let diag = Some(vec![
          format!("Trying to test output {:?}", expected),
          format!(
            "But component did not produce any more output. Component produced {} total packets.",
            test.actual.len()
          ),
        ]);
        test_block.add_test(|| false, prefix("stream_length"), diag);
        break;
      }
      let actual = &test.actual[j];
      let result = actual.port == expected.port;
      let diag = Some(vec![
        format!("Actual: {}", actual.port),
        format!("Expected: {}", expected.port),
      ]);
      test_block.add_test(move || result, prefix(&format!("output[{}]", expected.port)), diag);

      if let Some(value) = &expected.payload.value {
        let actual_payload = actual.payload.clone();

        let actual_value: Result<serde_value::Value, Error> = actual_payload
          .deserialize()
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

        info!(i,j,actual=?actual_value, "actual");
        info!(i,j,expected=?expected_value, "expected");
        test_block.add_test(
          move || match actual_value {
            Ok(val) => eq(val, expected_value),
            Err(_e) => false,
          },
          prefix("payload"),
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
          prefix("error_kind"),
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
          prefix("error_message"),
          diag,
        );
      }
    }
    let num_tested = test.outputs.len();
    let mut missed = vec![];
    for i in num_tested..test.actual.len() {
      if let Some(output) = test.actual.get(i) {
        if !matches!(output.payload, MessageTransport::Signal(_)) {
          debug!(?output, "test missed");
          missed.push(output);
        }
      }
    }
    let num_missed = missed.len();
    test_block.add_test(
      move || num_missed == 0,
      prefix("total_outputs"),
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
