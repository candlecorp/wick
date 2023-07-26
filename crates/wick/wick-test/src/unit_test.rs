use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_value::Value;
use wick_config::config::test_case::TestCase;
use wick_packet::{InherentData, Packet, PacketStream, RuntimeConfig};

use crate::error::AssertionFailure;
use crate::utils::gen_packet;
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

  pub(crate) fn check_next(&mut self, expected: Packet) -> Result<(), TestError> {
    let packets = self
      .actual
      .get_mut(expected.port())
      .ok_or(TestError::InvalidPort(expected.port().to_owned()))?;

    #[allow(clippy::never_loop)]
    while let Some(actual) = packets.pop_front() {
      if actual.port() != expected.port() {
        let a = AssertionFailure::Name(expected.port().to_owned(), actual.port().to_owned());
        return Err(TestError::Assertion(expected, actual, a));
      }

      if actual.flags() != expected.flags() {
        let a = AssertionFailure::Flags(expected.flags(), actual.flags());
        return Err(TestError::Assertion(expected, actual, a));
      }

      match (actual.has_data(), expected.has_data()) {
        (true, false) => return Err(TestError::Assertion(expected, actual, AssertionFailure::ExpectedNoData)),
        (false, true) => return Err(TestError::Assertion(expected, actual, AssertionFailure::ActualNoData)),
        (false, false) => return Ok(()),
        _ => {}
      }

      let actual_value: Value = actual
        .clone()
        .decode()
        .map_err(|e| TestError::Deserialization(e.to_string()))?;
      let expected_value: Value = expected
        .clone()
        .decode()
        .map_err(|e| TestError::Deserialization(e.to_string()))?;

      debug!(actual=?actual_value, expected=?expected_value, "test:packet");
      if !eq(&actual_value, &expected_value) {
        let a = AssertionFailure::Payload(expected_value, actual_value);
        return Err(TestError::Assertion(actual, expected, a));
      }

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

fn eq(left: &Value, right: &Value) -> bool {
  promote_val(left) == promote_val(right)
}

fn promote_val(val: &Value) -> Value {
  match val {
    Value::U8(n) => Value::U64((*n).into()),
    Value::U16(n) => Value::U64((*n).into()),
    Value::U32(n) => Value::U64((*n).into()),
    Value::I8(n) => Value::I64((*n).into()),
    Value::I16(n) => Value::I64((*n).into()),
    Value::I32(n) => Value::I64((*n).into()),
    Value::F32(n) => Value::F64((*n).into()),
    Value::Char(n) => Value::String((*n).into()),
    x => x.clone(),
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
      packets.push(gen_packet(packet, root_config, op_config)?);
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
