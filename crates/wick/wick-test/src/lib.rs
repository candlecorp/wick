// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
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
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

use std::time::{SystemTime, UNIX_EPOCH};

use flow_component::SharedComponent;
use serde_value::Value;
use tap_harness::{TestBlock, TestRunner};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use wick_config::config::TestCase;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketPayload, PacketStream};

use self::error::TestError;
use crate::utils::gen_packet;

pub mod error;
pub use error::TestError as Error;
mod utils;

#[macro_use]
extern crate tracing;

#[derive(Debug)]
#[must_use]
pub struct TestSuite<'a> {
  tests: Vec<UnitTest<'a>>,
  name: String,
  filters: Vec<String>,
}

impl<'a> Default for TestSuite<'a> {
  fn default() -> Self {
    Self::new("Test")
  }
}

impl<'a> TestSuite<'a> {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      tests: Vec::new(),
      name: name.as_ref().to_owned(),
      filters: Vec::new(),
    }
  }

  pub fn from_test_cases<'b>(tests: &'b [TestCase]) -> Self
  where
    'b: 'a,
  {
    let defs: Vec<UnitTest<'b>> = tests
      .iter()
      .map(|test| UnitTest {
        test,
        actual: Vec::new(),
      })
      .collect();
    Self {
      tests: defs,
      ..Default::default()
    }
  }

  pub fn get_tests<'b>(&'a mut self) -> Vec<&'a mut UnitTest<'b>>
  where
    'a: 'b,
  {
    let filters = &self.filters;
    if !filters.is_empty() {
      self
        .tests
        .iter_mut()
        .filter(|test| filters.iter().any(|filter| get_description(test).contains(filter)))
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

  pub async fn run(
    &'a mut self,
    collection_id: Option<&str>,
    collection: SharedComponent,
  ) -> Result<TestRunner, TestError> {
    let name = self.name.clone();
    let tests = self.get_tests();
    run_test(name, tests, collection_id, collection).await
  }
}

#[derive(Debug, Clone)]
pub struct UnitTest<'a> {
  pub test: &'a TestCase,
  pub actual: Vec<Packet>,
}

pub(crate) fn get_payload(test: &UnitTest) -> (PacketStream, Option<InherentData>) {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  for packet in &test.test.inputs {
    debug!("Test input for port {:?}", packet);
    tx.send(
      gen_packet(packet)
        .map_err(|e| wick_packet::Error::General(format!("could not convert test packet to real packet: {}", e))),
    )
    .unwrap();
  }
  let stream = PacketStream::new(Box::new(UnboundedReceiverStream::new(rx)));
  if let Some(inherent) = test.test.inherent {
    if let Some(seed) = inherent.seed {
      return (
        stream,
        Some(InherentData::new(
          seed,
          SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap(),
        )),
      );
    }
  }
  (stream, None)
}

#[must_use]
pub fn get_description(test: &UnitTest) -> String {
  format!("{}: test operation '{}'", test.test.name, test.test.operation)
}

pub async fn run_test<'a, 'b>(
  name: impl AsRef<str> + Sync + Send,
  defs: Vec<&'a mut UnitTest<'a>>,
  id: Option<&'b str>,
  collection: SharedComponent,
) -> Result<TestRunner, Error> {
  let mut harness = TestRunner::new(Some(name));

  for (i, def) in defs.into_iter().enumerate() {
    let entity = id.map_or_else(
      || Entity::local(&def.test.operation),
      |id| Entity::operation(id, &def.test.operation),
    );
    let block = run_unit(i, def, entity, collection.clone()).await?;
    harness.add_block(block);
  }

  harness.run();
  Ok(harness)
}

#[allow(clippy::too_many_lines)]
async fn run_unit<'a>(
  i: usize,
  def: &'a mut UnitTest<'a>,
  entity: Entity,
  collection: SharedComponent,
) -> Result<TestBlock, TestError> {
  let (stream, inherent) = get_payload(def);
  let test_name = get_description(def);
  let mut test_block = TestBlock::new(Some(test_name.clone()));
  let prefix = |msg: &str| format!("{}: {}", test_name, msg);

  trace!(i, %entity, "invoke");
  let invocation = Invocation::new(Entity::test(&test_name), entity, inherent);
  let result = collection
    .handle(invocation, stream, None, std::sync::Arc::new(|_, _, _, _| panic!()))
    .await
    .map_err(|e| Error::InvocationFailed(e.to_string()));

  if let Err(e) = result {
    test_block.add_test(
      || false,
      prefix("invocation"),
      Some(vec![format!("Invocation failed: {}", e)]),
    );
    return Ok(test_block);
  }

  let stream = result.unwrap();

  let outputs: Vec<_> = stream.filter(|msg| msg.is_ok()).map(|msg| msg.unwrap()).collect().await;
  def.actual = outputs;
  let mut diagnostics = vec!["Output: ".to_owned()];
  let mut output_lines: Vec<_> = def.actual.iter().map(|o| format!("{:?}", o)).collect();
  diagnostics.append(&mut output_lines);
  test_block.add_diagnostic_messages(diagnostics);

  for (j, expected) in def.test.outputs.iter().cloned().enumerate() {
    if j >= def.actual.len() {
      let diag = Some(vec![
        format!("Trying to test output {:?}", expected),
        format!(
          "But component did not produce any more output. Component produced {} total packets.",
          def.actual.len()
        ),
      ]);
      test_block.add_test(|| false, prefix("stream_length"), diag);
      break;
    }
    let actual = &def.actual[j];
    let result = actual.port() == expected.port();
    let diag = diag_compare(actual.port(), expected.port());
    test_block.add_test(
      move || result,
      prefix(&format!("correct port? ('{}' == '{}')", actual.port(), expected.port())),
      diag,
    );
    let expected = gen_packet(&expected)?;

    let actual_payload = actual.payload.clone();

    if actual.flags() > 0 && actual_payload.bytes().map_or(true, |v| v.is_empty()) {
      let diagnostic = diag_compare(diag_packet(actual), diag_packet(&expected));

      debug!(i,j,actual=?actual, "actual");
      debug!(i,j,expected=?expected, "expected");
      let e_inner = expected.clone();
      let a_inner = actual.clone();
      test_block.add_test(
        move || packet_eq(a_inner, e_inner),
        prefix("actual raw packet == expected raw packet?"),
        diagnostic,
      );
    } else {
      let actual_value: Result<Value, Error> = actual_payload
        .deserialize()
        .map_err(|e| Error::ConversionFailed(e.to_string()));
      let expected_value: Result<Value, Error> = expected
        .deserialize()
        .map_err(|e| Error::ConversionFailed(e.to_string()));

      let diagnostic = diag_compare(diag_value(&actual_value), diag_value(&expected_value));

      debug!(i,j,actual=?actual_value, "actual");
      debug!(i,j,expected=?expected_value, "expected");
      test_block.add_test(
        move || match (actual_value, expected_value) {
          (Ok(actual), Ok(expected)) => eq(actual, expected),
          (Err(actual), Err(expected)) => err_eq(actual, expected),
          _ => false,
        },
        prefix("actual deserialized payload == expected deserialized payload?"),
        diagnostic,
      );
    }
  }

  let num_tested = def.test.outputs.len();
  let mut missed = vec![];

  for i in num_tested..def.actual.len() {
    if let Some(output) = def.actual.get(i) {
      if output.is_done() {
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

  Ok(test_block)
}

fn diag_value(result: &Result<Value, TestError>) -> String {
  match result {
    Ok(v) => format!("{:?}", v),
    Err(e) => format!("Could not deserialize payload, message was : {}", e),
  }
}

fn diag_packet(packet: &Packet) -> String {
  match &packet.payload {
    PacketPayload::Ok(v) => format!("Ok: {:?} (flags: {:08b})", v, packet.flags()),
    PacketPayload::Err(e) => format!("Err: {} (flags: {:08b})", e.msg(), packet.flags()),
  }
}

fn diag_compare(actual: impl AsRef<str>, expected: impl AsRef<str>) -> Option<Vec<String>> {
  Some(vec![
    format!("Actual: {}", actual.as_ref()),
    format!("Expected: {}", expected.as_ref()),
  ])
}

fn eq(left: Value, right: Value) -> bool {
  promote_val(left) == promote_val(right)
}

#[allow(clippy::needless_pass_by_value)]
fn packet_eq(left: Packet, right: Packet) -> bool {
  left == right
}

#[allow(clippy::needless_pass_by_value)]
fn err_eq(left: Error, right: Error) -> bool {
  left == right
}

fn promote_val(val: Value) -> Value {
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

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use anyhow::Result;
  use test_native_component::NativeComponent;
  use wick_config::WickConfiguration;

  use super::*;

  fn get_component() -> SharedComponent {
    Arc::new(NativeComponent::default())
  }

  #[test_logger::test(tokio::test)]
  async fn test_basic() -> Result<()> {
    let config = include_str!("../tests/manifests/test.yaml");
    let config = WickConfiguration::from_yaml(config, &None)?.try_test_config()?;
    let mut unit_tests = TestSuite::from_test_cases(config.tests());
    let results = unit_tests.run(None, get_component()).await;
    assert!(results.is_ok());
    let results = results.unwrap();
    let lines = results.get_tap_lines();
    println!("{}", lines.join("\n"));
    assert_eq!(results.num_failed(), 0);

    Ok(())
  }
}
