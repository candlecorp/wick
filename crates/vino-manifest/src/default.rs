use std::borrow::Cow;

/// Parse a JSON template.
pub fn parse_default(json_str: &str) -> Result<serde_json::Value, serde_json::Error> {
  serde_json::from_str(json_str)
}

/// The string to replace in the JSON template.
pub const ERROR_STR: &str = "$ERROR";

/// Render the JSON template while replacing any instance of [ERROR_STR] with the passed message.
pub fn process_default<'a>(
  obj: Cow<'a, serde_json::Value>,
  message: &str,
) -> Result<Cow<'a, serde_json::Value>, serde_json::Error> {
  let result = if obj.is_string() {
    let s = obj.as_str().unwrap();
    if s.contains(ERROR_STR) {
      Cow::Owned(serde_json::Value::String(s.replace(ERROR_STR, message)))
    } else {
      obj
    }
  } else if obj.is_object() {
    let object = obj.as_object().unwrap();
    let mut new_values = vec![];
    for (k, v) in object.iter() {
      let cow_v = Cow::Borrowed(v);
      let new_v = process_default(cow_v.clone(), message)?;
      if cow_v != new_v {
        new_values.push((k, new_v));
      }
    }
    if !new_values.is_empty() {
      let mut object = object.clone();
      for (k, v) in new_values {
        object.insert(k.clone(), v.into_owned());
      }
      Cow::Owned(serde_json::Value::Object(object))
    } else {
      obj
    }
  } else if obj.is_array() {
    let array = obj.as_array().unwrap();
    let mut new_values = vec![];

    #[allow(clippy::needless_range_loop)]
    for i in 0..array.len() {
      let cow_v = Cow::Borrowed(&array[i]);
      let new_v = process_default(cow_v.clone(), message)?;
      if cow_v != new_v {
        new_values.push((i, new_v));
      }
    }
    if !new_values.is_empty() {
      let mut array = array.clone();
      for (i, v) in new_values {
        array[i] = v.into_owned();
      }
      Cow::Owned(serde_json::Value::Array(array))
    } else {
      obj
    }
  } else {
    obj
  };

  Ok(result)
}

#[cfg(test)]
mod tests {
  use anyhow::Result as TestResult;
  use pretty_assertions::assert_eq as equals;
  use serde::{
    Deserialize,
    Serialize,
  };
  use serde_json::json;

  use super::*;

  fn process_json(json_str: &str, err: &str, assert_same: bool) -> TestResult<serde_json::Value> {
    let json = parse_default(json_str)?;
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    if assert_same {
      equals!(Cow::Borrowed(&json), new_json);
    }
    Ok(new_json.into_owned())
  }

  #[test_env_log::test]
  fn test_cow_impl() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let json = parse_default(json_str)?;

    let err = "This is my error message";
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    equals!(new_json.into_owned(), json!(format!("Error: {}", err)));

    let err = "This another error message";
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    equals!(new_json.into_owned(), json!(format!("Error: {}", err)));

    Ok(())
  }

  #[test_env_log::test]
  fn test_string_no_sub() -> TestResult<()> {
    let json_str = r#"
    "My default"
    "#;

    let real = process_json(json_str, "This is my error message", true)?;

    equals!(real, "My default");

    Ok(())
  }

  #[test_env_log::test]
  fn test_string_with_sub() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let err = "This is my error message";

    let real = process_json(json_str, err, false)?;

    equals!(real, format!("Error: {}", err));

    Ok(())
  }

  #[test_env_log::test]
  fn test_obj_no_sub() -> TestResult<()> {
    let json_str = r#"
    {"method":"GET"}
    "#;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Request {
      method: String,
    }

    let err = "This is my error message";

    let real = process_json(json_str, err, true)?;

    equals!(
      real,
      serde_json::to_value(Request {
        method: "GET".to_owned()
      })?
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_obj_with_sub() -> TestResult<()> {
    let json_str = r#"
    {"method":"GET: $ERROR"}
    "#;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Request {
      method: String,
    }

    let err = "This is my error message";

    let real = process_json(json_str, err, false)?;

    equals!(
      real,
      serde_json::to_value(Request {
        method: format!("GET: {}", err)
      })?
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_arr_no_sub() -> TestResult<()> {
    let json_str = r#"
    ["this", "that"]
    "#;

    let err = "This is my error message";

    let real = process_json(json_str, err, true)?;

    equals!(
      real,
      serde_json::to_value(vec!["this".to_owned(), "that".to_owned()])?
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_arr_with_sub() -> TestResult<()> {
    let json_str = r#"
    ["this", "$ERROR"]
    "#;

    let err = "This is my error message";

    let real = process_json(json_str, err, false)?;

    equals!(
      real,
      serde_json::to_value(vec!["this".to_owned(), err.to_owned()])?
    );

    Ok(())
  }
}
