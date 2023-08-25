use serde_json::Value;

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

pub(super) fn assert_contains(expected: &Value, actual: &Value) -> Result<(), ContainsError> {
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

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::*;

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
  fn test_contains_equals(#[case] expected: Value, #[case] actual: Value) -> Result<()> {
    assert_eq!(assert_contains(&expected, &actual), Ok(()));

    Ok(())
  }

  #[rstest::rstest]
  #[case(json!({"a": 1, "b": 2, }), json!({ "a": 1, "c": 3, }), ContainsError::MissingKey(json!({ "a": 1, "c": 3, }), "b".into()))]
  #[case(json!({"a": 1, "b": 2, }), json!({ "a": 1, "b": 100, }), ContainsError::Mismatch(json!(2), json!(100)))]
  #[case(json!({"a": 1, "b": {"c":2}, }), json!({ "a": 1, "b": 100, }), ContainsError::NotAnObject(json!(100)))]
  fn test_contains_not_ok(#[case] expected: Value, #[case] actual: Value, #[case] error: ContainsError) -> Result<()> {
    assert_eq!(assert_contains(&expected, &actual), Err(error));

    Ok(())
  }
}
