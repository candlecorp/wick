use std::borrow::Cow;

use crate::dev::prelude::*;

pub(crate) fn parse_default(json_str: &str) -> Result<serde_json::Value, serde_json::Error> {
  serde_json::from_str(json_str)
}

pub(crate) const ERROR_STR: &str = "$ERROR";

pub(crate) fn process_default<'a>(
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

pub(crate) fn make_default_transport(json: &serde_json::Value, message: &str) -> MessageTransport {
  process_default(Cow::Borrowed(json), message).map_or(
    MessageTransport::Error("Error processing default value".to_owned()),
    |result| {
      mp_serialize(&result).map_or(
        MessageTransport::Error("Error serializing default value".to_owned()),
        |bytes| MessageTransport::MessagePack(bytes),
      )
    },
  )
}

#[cfg(test)]
mod tests {
  use serde::{
    Deserialize,
    Serialize,
  };

  use super::*;
  use crate::test::prelude::*;

  fn process_json<'de, T: Deserialize<'de>>(
    json_str: &str,
    err: &str,
    assert_same: bool,
  ) -> TestResult<T> {
    let json = parse_default(json_str)?;
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    if assert_same {
      equals!(Cow::Borrowed(&json), new_json);
    }
    let real: T = messagepack_roundtrip(&new_json)?;
    Ok(real)
  }

  fn messagepack_roundtrip<'de, IN: Serialize, OUT: Deserialize<'de>>(
    input: &IN,
  ) -> TestResult<OUT> {
    Ok(mp_deserialize(&mp_serialize(input)?)?)
  }

  #[test_env_log::test]
  fn test_cow_impl() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let json = parse_default(json_str)?;

    let err = "This is my error message";
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    let real: String = messagepack_roundtrip(&new_json)?;
    equals!(real, format!("Error: {}", err));

    let err = "This another error message";
    let new_json = process_default(Cow::Borrowed(&json), err)?;
    let real: String = messagepack_roundtrip(&new_json)?;
    equals!(real, format!("Error: {}", err));

    Ok(())
  }

  #[test_env_log::test]
  fn test_to_transport() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let json = parse_default(json_str)?;

    let err = "This is my error message";
    let message = make_default_transport(&json, err);

    equals!(
      message,
      MessageTransport::MessagePack(mp_serialize(format!("Error: {}", err))?)
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_string_no_sub() -> TestResult<()> {
    let json_str = r#"
    "My default"
    "#;

    let real: String = process_json(json_str, "This is my error message", true)?;

    equals!(real, "My default");

    Ok(())
  }

  #[test_env_log::test]
  fn test_string_with_sub() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let err = "This is my error message";

    let real: String = process_json(json_str, err, false)?;

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

    let real: Request = process_json(json_str, err, true)?;

    equals!(
      real,
      Request {
        method: "GET".to_owned()
      }
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

    let real: Request = process_json(json_str, err, false)?;

    equals!(
      real,
      Request {
        method: format!("GET: {}", err)
      }
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_arr_no_sub() -> TestResult<()> {
    let json_str = r#"
    ["this", "that"]
    "#;

    let err = "This is my error message";

    let real: Vec<String> = process_json(json_str, err, true)?;

    equals!(real, vec!["this".to_owned(), "that".to_owned()]);

    Ok(())
  }

  #[test_env_log::test]
  fn test_arr_with_sub() -> TestResult<()> {
    let json_str = r#"
    ["this", "$ERROR"]
    "#;

    let err = "This is my error message";

    let real: Vec<String> = process_json(json_str, err, false)?;

    equals!(real, vec!["this".to_owned(), err.to_owned()]);

    Ok(())
  }
}
