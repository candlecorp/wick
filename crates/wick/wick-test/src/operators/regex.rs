use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum RegexError {
  BadRegex { regex: String, error: String },
  RegexNotString { value: Value },
  ValueNotString { value: Value },
  NoMatches { regex: String, value: Value },
}

impl std::fmt::Display for RegexError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RegexError::BadRegex { regex, error } => write!(f, "could not compile regex '{}' : {}", regex, error),
      RegexError::NoMatches { regex, value } => write!(f, "regex '{}' does match {}", regex, value),
      RegexError::RegexNotString { value } => write!(f, "regex must be a simple string value, got: {}", value),
      RegexError::ValueNotString { value } => write!(f, "can not match regex against non-string value: {}", value),
    }
  }
}

fn to_str(value: &Value) -> Option<&str> {
  match value {
    Value::String(s) => Some(s),
    _ => None,
  }
}

pub(super) fn assert_matches(expected: &Value, actual: &Value) -> Result<(), RegexError> {
  let exp_str = to_str(expected).ok_or_else(|| RegexError::RegexNotString {
    value: expected.clone(),
  })?;
  let act_str = to_str(actual).ok_or_else(|| RegexError::ValueNotString { value: actual.clone() })?;

  let regex = regex::Regex::new(exp_str).map_err(|e| RegexError::BadRegex {
    regex: exp_str.to_owned(),
    error: e.to_string(),
  })?;

  if regex.is_match(act_str) {
    Ok(())
  } else {
    Err(RegexError::NoMatches {
      regex: exp_str.to_owned(),
      value: actual.clone(),
    })
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::*;

  #[rstest::rstest]
  #[case(
    json!("\\w+ \\w+"),
    json!("hello world")
  )]
  fn test_ok(#[case] expected_value: Value, #[case] actual: Value) -> Result<()> {
    assert_eq!(assert_matches(&expected_value, &actual,), Ok(()));

    Ok(())
  }

  #[rstest::rstest]
  #[case(json!({"a": 1, "b": 2, }), json!({ "a": 1, "b": 100, }), RegexError::RegexNotString { value: json!({"a": 1, "b": 2, }) })]
  #[case(json!("regex"), json!({ "a": 1, "b": 100, }), RegexError::ValueNotString { value: json!({ "a": 1, "b": 100, }) })]
  #[case(json!("this"), json!("that"), RegexError::NoMatches { regex: "this".to_owned(), value:json!("that")  } )]
  #[case(json!("*"), json!("that"), RegexError::BadRegex { regex: "*".to_owned(), error: "regex parse error:\n    *\n    ^\nerror: repetition operator missing expression".to_owned() })]
  fn test_not_ok(#[case] expected: Value, #[case] actual: Value, #[case] error: RegexError) -> Result<()> {
    assert_eq!(assert_matches(&expected, &actual), Err(error));

    Ok(())
  }
}
