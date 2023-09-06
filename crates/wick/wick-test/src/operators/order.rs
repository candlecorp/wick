use serde_json::Value;
use wick_config::config::test_case::AssertionOperator;

#[derive(Debug, PartialEq)]
pub struct OrderingError {
  pub expected: Value,
  pub actual: Value,
  pub kind: Ordering,
}

impl std::fmt::Display for OrderingError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "comparison failed, expected {} {} {}",
      self.expected, self.kind, self.actual
    )
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ordering {
  ExpectedLessThan,
  ExpectedGreaterThan,
  ExpectedEqual,
  TypeMismatch,
}

impl std::fmt::Display for Ordering {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Ordering::ExpectedLessThan => f.write_str("to be less than"),
      Ordering::ExpectedGreaterThan => f.write_str("to be greater than"),
      Ordering::TypeMismatch => f.write_str("to loosely match the type of"),
      Ordering::ExpectedEqual => f.write_str("to equal"),
    }
  }
}

enum Num {
  I(i64),
  U(u64),
  F(f64),
}

#[allow(clippy::option_if_let_else)]
impl From<&serde_json::Number> for Num {
  fn from(n: &serde_json::Number) -> Self {
    if let Some(n) = n.as_i64() {
      Num::I(n)
    } else if let Some(n) = n.as_u64() {
      Num::U(n)
    } else if let Some(n) = n.as_f64() {
      Num::F(n)
    } else {
      unreachable!()
    }
  }
}

#[allow(clippy::cast_lossless)]
impl PartialEq for Num {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Num::I(a), Num::I(b)) => a == b,
      (Num::I(a), Num::U(b)) => (*a as i128) == (*b as i128),
      (Num::I(a), Num::F(b)) => (*a as f64) == *b,
      (Num::U(a), Num::I(b)) => (*a as i128) == (*b as i128),
      (Num::U(a), Num::U(b)) => a == b,
      (Num::U(a), Num::F(b)) => (*a as f64) == *b,
      (Num::F(a), Num::I(b)) => *a == (*b as f64),
      (Num::F(a), Num::U(b)) => *a == (*b as f64),
      (Num::F(a), Num::F(b)) => *a == *b,
    }
  }
}

#[allow(clippy::cast_lossless)]
impl PartialOrd for Num {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (Num::I(a), Num::I(b)) => a.partial_cmp(b),
      (Num::I(a), Num::U(b)) => (*a as i128).partial_cmp(&(*b as i128)),
      (Num::I(a), Num::F(b)) => (*a as f64).partial_cmp(b),
      (Num::U(a), Num::I(b)) => (*a as i128).partial_cmp(&(*b as i128)),
      (Num::U(a), Num::U(b)) => a.partial_cmp(b),
      (Num::U(a), Num::F(b)) => (*a as f64).partial_cmp(b),
      (Num::F(a), Num::I(b)) => a.partial_cmp(&(*b as f64)),
      (Num::F(a), Num::U(b)) => a.partial_cmp(&(*b as f64)),
      (Num::F(a), Num::F(b)) => a.partial_cmp(b),
    }
  }
}

pub(super) fn assert_order(op: AssertionOperator, expected: &Value, actual: &Value) -> Result<(), OrderingError> {
  compare(op, expected, actual).map_err(|kind| OrderingError {
    expected: expected.clone(),
    actual: actual.clone(),
    kind,
  })
}

pub(super) fn compare(op: AssertionOperator, comparison: &Value, actual: &Value) -> Result<(), Ordering> {
  match (comparison, actual) {
    (Value::Number(compare_num), Value::Number(act_num)) => {
      let compare_num: Num = compare_num.into();
      let act_num: Num = act_num.into();
      match op {
        AssertionOperator::Equals => {
          if compare_num == act_num {
            Ok(())
          } else {
            Err(Ordering::ExpectedEqual)
          }
        }
        AssertionOperator::LessThan => {
          if act_num < compare_num {
            Ok(())
          } else {
            Err(Ordering::ExpectedLessThan)
          }
        }
        AssertionOperator::GreaterThan => {
          if act_num > compare_num {
            Ok(())
          } else {
            Err(Ordering::ExpectedGreaterThan)
          }
        }
        _ => unreachable!(),
      }
    }
    (_, _) => {
      if op == AssertionOperator::Equals {
        if comparison == actual {
          Ok(())
        } else {
          Err(Ordering::ExpectedEqual)
        }
      } else {
        Err(Ordering::TypeMismatch)
      }
    }
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;
  use wick_config::config::test_case::AssertionOperator;

  use super::*;

  #[rstest::rstest]
  #[case(AssertionOperator::GreaterThan,json!(1),json!(2),)]
  #[case(AssertionOperator::GreaterThan,json!(1.1),json!(2),)]
  #[case(AssertionOperator::GreaterThan,json!(1.1),json!(1.2),)]
  #[case(AssertionOperator::GreaterThan,json!(-1.1),json!(-1),)]
  #[case(AssertionOperator::LessThan,json!(4),json!(2),)]
  #[case(AssertionOperator::LessThan,json!(4.1),json!(2),)]
  #[case(AssertionOperator::LessThan,json!(4.4),json!(4.2),)]
  #[case(AssertionOperator::LessThan,json!(-1.1),json!(-2),)]
  #[case(AssertionOperator::Equals,json!(1),json!(1),)]
  #[case(AssertionOperator::Equals,json!(1.0),json!(1),)]
  #[case(AssertionOperator::Equals,json!(1),json!(1.0),)]
  #[case(AssertionOperator::Equals,json!(i64::MAX),json!(i64::MAX as u64),)]
  #[case(AssertionOperator::Equals,json!(i64::MAX as u64),json!(i64::MAX),)]
  fn test_ok(#[case] op: AssertionOperator, #[case] expected_value: Value, #[case] actual: Value) -> Result<()> {
    assert_eq!(compare(op, &expected_value, &actual,), Ok(()));

    Ok(())
  }

  #[rstest::rstest]
  #[case(AssertionOperator::LessThan,json!({"a": 1, "b": 2, }), json!({ "a": 1, "b": 100, }), Ordering::TypeMismatch)]
  fn test_not_ok(
    #[case] op: AssertionOperator,
    #[case] expected: Value,
    #[case] actual: Value,
    #[case] error: Ordering,
  ) -> Result<()> {
    assert_eq!(compare(op, &expected, &actual), Err(error));

    Ok(())
  }
}
