mod contains;
mod order;
mod regex;

use json_dotpath::DotPaths;
use serde_json::Value;
use wick_config::config::test_case::AssertionOperator;
use wick_packet::{Packet, PacketExt};

pub(crate) use self::contains::ContainsError;
pub(crate) use self::order::OrderingError;
pub(crate) use self::regex::RegexError;
use crate::assertion_packet::TestKind;
use crate::error::AssertionFailure;
use crate::TestError;

pub(crate) fn assert_packet(expected: &TestKind, actual: Packet) -> Result<(), TestError> {
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
        let actual_value = if let Some(path) = &assertion.path {
          match actual_value.dot_get::<Value>(path) {
            Ok(Some(v)) => v,
            Ok(None) | Err(_) => {
              error!(
                "could not find value at path '{}' in packet data {}",
                path, actual_value
              );
              return Err(TestError::DotPath(path.clone()));
            }
          }
        } else {
          actual_value.clone()
        };
        debug!(op=?assertion.operator, actual=?actual_value, expected=?assertion.value, "test:packet");

        match assertion.operator {
          AssertionOperator::Equals | AssertionOperator::LessThan | AssertionOperator::GreaterThan => {
            if let Err(e) = order::assert_order(assertion.operator, &assertion.value, &actual_value) {
              return Err(TestError::Assertion(
                expected.clone(),
                actual,
                AssertionFailure::Ordering(e),
              ));
            }
          }
          AssertionOperator::Regex => {
            if let Err(e) = regex::assert_matches(&assertion.value, &actual_value) {
              return Err(TestError::Assertion(
                expected.clone(),
                actual,
                AssertionFailure::Regex(e),
              ));
            }
          }
          AssertionOperator::Contains => {
            if let Err(e) = contains::assert_contains(&assertion.value, &actual_value) {
              return Err(TestError::Assertion(
                expected.clone(),
                actual,
                AssertionFailure::Contains(e),
              ));
            }
          }
        }
      }
    }
  };

  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;
  use wick_config::config::test_case::AssertionOperator;
  use wick_packet::Packet;

  use super::*;
  use crate::assertion_packet::{Assertion, AssertionDef, TestKind};
  use crate::error::{AssertionFailure, TestError};

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
        operator: AssertionOperator::Contains,
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
        operator: AssertionOperator::Contains,
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
