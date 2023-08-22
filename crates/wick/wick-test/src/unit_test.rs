use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use wick_config::config::test_case::TestCase;
use wick_packet::{InherentData, Packet, PacketStream, RuntimeConfig};

use crate::assertion_packet::{TestKind, ToPacket};
use crate::error::AssertionFailure;
use crate::TestError;

#[derive(Debug, Clone)]
pub struct UnitTest<'a> {
  pub test: &'a TestCase,
  actual: HashMap<String, VecDeque<Packet>>,
}

impl<'a> UnitTest<'a> {
  pub(crate) fn new(test: &'a TestCase) -> Self {
    Self {
      test,
      actual: HashMap::new(),
    }
  }

  pub fn set_actual(&mut self, actual: Vec<Packet>) {
    for packet in actual {
      self
        .actual
        .entry(packet.port().to_owned())
        .or_default()
        .push_back(packet);
    }
  }

  pub(crate) fn check_next(&mut self, expected: &TestKind) -> Result<(), TestError> {
    let packets = self
      .actual
      .get_mut(expected.port())
      .ok_or(TestError::InvalidPort(expected.port().to_owned()))?;

    #[allow(clippy::never_loop)]
    while let Some(actual) = packets.pop_front() {
      assert_packet(expected, actual)?;

      return Ok(());
    }
    Ok(())
  }

  pub(crate) fn finalize(&mut self, _explicit_done: &HashSet<String>) -> Result<(), Vec<Packet>> {
    let mut with_data = Vec::new();
    let packets = self.actual.drain().collect::<Vec<_>>();
    for (port, packets) in packets {
      for packet in packets {
        if packet.is_done() {
          debug!(port, "test: received done packet without assertion, ignoring");
          continue;
        }
        with_data.push(packet);
        break;
      }
    }
    if with_data.is_empty() {
      Ok(())
    } else {
      Err(with_data)
    }
  }
}

fn assert_packet(expected: &TestKind, actual: Packet) -> Result<(), TestError> {
  if actual.port() != expected.port() {
    let a = AssertionFailure::Name(expected.port().to_owned(), actual.port().to_owned());
    return Err(TestError::Assertion(expected.clone(), actual, a));
  }

  if actual.flags() != expected.flags() {
    let a = AssertionFailure::Flags(expected.flags(), actual.flags());
    return Err(TestError::Assertion(expected.clone(), actual, a));
  }

  match (actual.has_data(), expected.has_data()) {
    (true, false) => {
      return Err(TestError::Assertion(
        expected.clone(),
        actual,
        AssertionFailure::ExpectedNoData,
      ))
    }
    (false, true) => {
      return Err(TestError::Assertion(
        expected.clone(),
        actual,
        AssertionFailure::ActualNoData,
      ))
    }
    (false, false) => return Ok(()),
    _ => {}
  }

  let actual_value: Value = actual
    .clone()
    .decode()
    .map_err(|e| TestError::Deserialization(e.to_string()))?;

  match expected {
    TestKind::Exact(expected_packet) => {
      let expected_value: Value = expected_packet
        .clone()
        .decode()
        .map_err(|e| TestError::Deserialization(e.to_string()))?;

      debug!(actual=?actual_value, expected=?expected_value, "test:packet");
      if actual_value != expected_value {
        let a = AssertionFailure::Payload(expected_value, actual_value);
        return Err(TestError::Assertion(expected.clone(), actual, a));
      }
    }
    TestKind::Assertion(assertion_def) => {
      for assertion in &assertion_def.assertions {
        match assertion.operator {
          wick_config::config::test_case::AssertionOperator::Equals => todo!(),
          wick_config::config::test_case::AssertionOperator::LessThan => todo!(),
          wick_config::config::test_case::AssertionOperator::GreaterThan => todo!(),
          wick_config::config::test_case::AssertionOperator::Regex => todo!(),
          wick_config::config::test_case::AssertionOperator::Contains => {
            debug!(actual=?actual_value, expected=?assertion.value, "test:packet");

            if let Err(e) = assert_contains(&assertion.value, &actual_value) {
              let a = AssertionFailure::Contains(e);
              return Err(TestError::Assertion(expected.clone(), actual, a));
            }
          }
        }
      }
    }
  };

  Ok(())
}

#[derive(Debug, PartialEq)]
pub enum ContainsError {
  NotAnArray(Value),
  NotAnObject(Value),
  MissingKey(Value, String),
  MissingIndex(Value, usize),
  Mismatch(Value, Value),
}

impl std::fmt::Display for ContainsError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ContainsError::NotAnArray(v) => write!(f, "expected an array, got {:?}", v),
      ContainsError::NotAnObject(v) => write!(f, "expected an object, got {:?}", v),
      ContainsError::MissingKey(v, k) => write!(f, "expected key {:?} in object {:?}", k, v),
      ContainsError::MissingIndex(v, i) => write!(f, "expected index {:?} in array {:?}", i, v),
      ContainsError::Mismatch(expected, actual) => write!(f, "expected {:?} to equal {:?}", expected, actual),
    }
  }
}

fn assert_contains(expected: &Value, actual: &Value) -> Result<(), ContainsError> {
  match expected {
    Value::Array(v) => {
      if let Value::Array(v2) = actual {
        for (i, v) in v.iter().enumerate() {
          assert_contains(
            v,
            v2.get(i)
              .ok_or_else(|| ContainsError::MissingIndex(actual.clone(), i))?,
          )?;
        }
      } else {
        return Err(ContainsError::NotAnArray(actual.clone()));
      }

      Ok(())
    }
    Value::Object(v) => {
      if let Value::Object(v2) = actual {
        for (k, v) in v.iter() {
          assert_contains(
            v,
            v2.get(k)
              .ok_or_else(|| ContainsError::MissingKey(actual.clone(), k.clone()))?,
          )?;
        }
      } else {
        return Err(ContainsError::NotAnObject(actual.clone()));
      }
      Ok(())
    }
    _ => {
      if expected == actual {
        Ok(())
      } else {
        Err(ContainsError::Mismatch(expected.clone(), actual.clone()))
      }
    }
  }
}

pub(crate) fn get_payload(
  test: &UnitTest,
  root_config: Option<&RuntimeConfig>,
  op_config: Option<&RuntimeConfig>,
) -> Result<(PacketStream, InherentData, HashSet<String>), TestError> {
  let mut packets = Vec::new();
  let mut open_streams = HashSet::new();

  // need a Vec to store the order of seen ports so we can have repeatable tests.
  // HashSet doesn't guarantee order.
  let mut order = Vec::new();

  for packet in test.test.inputs() {
    // if we've never seen this port before, push it onto the order Vec.
    if !open_streams.contains(packet.port()) {
      order.push(packet.port().to_owned());
    }
    open_streams.insert(packet.port().to_owned());
  }

  let mut explicit_done = HashSet::new();

  if test.test.inputs().is_empty() {
    packets.push(Packet::no_input());
  } else {
    for packet in test.test.inputs() {
      let done = packet.flag().is_done();
      if done {
        explicit_done.insert(packet.port().to_owned());
        open_streams.remove(packet.port());
      } else if !open_streams.contains(packet.port()) {
        return Err(TestError::PacketsAfterDone(packet.port().to_owned()));
      }
      debug!(?packet, "test packet");
      packets.push(packet.to_packet(root_config, op_config)?);
    }

    // Add any missing "done" packets as a convenience to the test writer.
    // (in the order we saw the ports)
    for port in order {
      if open_streams.contains(&port) {
        debug!(input = port, "adding missing done packet for input");
        packets.push(Packet::done(port));
      }
    }
  }

  let (seed, timestamp) = test.test.inherent().map_or_else(
    || {
      (
        0,
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .try_into()
          .unwrap(),
      )
    },
    |inherent| {
      let seed = inherent.seed().unwrap_or(0);
      let timestamp = inherent.timestamp().unwrap_or(
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .try_into()
          .unwrap(),
      );
      (seed, timestamp)
    },
  );

  Ok((packets.into(), InherentData::new(seed, timestamp), explicit_done))
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;
  use wick_config::config::test_case::AssertionOperator;

  use super::*;
  use crate::assertion_packet::{Assertion, AssertionDef};

  #[rstest::rstest]
  #[case(
    json!({
    "a": 1,
    "b": 2,
    "c": 3,
    }),
    json!({
      "a": 1,
      "b": 2,
      "c": 3,
    })
  )]
  fn test_contains_equals(#[case] expected_value: Value, #[case] actual: Value) -> Result<()> {
    let actual = Packet::encode("...", actual);
    let expected = TestKind::Assertion(AssertionDef {
      port: actual.port().to_owned(),
      assertions: vec![Assertion {
        path: None,
        operator: AssertionOperator::Equals,
        value: expected_value.clone(),
      }],
    });

    assert_eq!(assert_packet(&expected, actual.clone(),), Ok(()));

    Ok(())
  }

  #[rstest::rstest]
  #[case(json!({"a": 1, "b": 2, }), json!({ "a": 1, "c": 3, }), ContainsError::MissingKey(json!({ "a": 1, "c": 3, }), "b".into()))]
  #[case(json!({"a": 1, "b": 2, }), json!({ "a": 1, "b": 100, }), ContainsError::Mismatch(json!(2), json!(100)))]
  #[case(json!({"a": 1, "b": {"c":2}, }), json!({ "a": 1, "b": 100, }), ContainsError::NotAnObject(json!(100)))]
  fn test_contains_not_ok(
    #[case] expected_value: Value,
    #[case] actual: Value,
    #[case] error: ContainsError,
  ) -> Result<()> {
    let actual = Packet::encode("...", actual);
    let expected = TestKind::Assertion(AssertionDef {
      port: actual.port().to_owned(),
      assertions: vec![Assertion {
        path: None,
        operator: AssertionOperator::Equals,
        value: expected_value.clone(),
      }],
    });

    assert_eq!(
      assert_packet(&expected, actual.clone(),),
      Err(TestError::Assertion(
        expected,
        actual,
        AssertionFailure::Contains(error)
      ))
    );

    Ok(())
  }
}
