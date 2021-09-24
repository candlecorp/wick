use vino_transport::{
  MessageTransport,
  Success,
  TransportMap,
};

use crate::Error;

/// Parse CLI arguments into a [TransportMap]
pub fn parse_args(args: &[String]) -> Result<TransportMap, Error> {
  let mut map = TransportMap::new();
  let mut iter = args.iter();
  while let Some(next) = iter.next() {
    if !next.starts_with("--") {
      return Err(Error::InvalidArgument(next.clone()));
    }
    let next = next.trim_start_matches("--");
    let (name, value) = split_arg(next);
    let value = match value {
      Some(value) => value,
      None => {
        let value = iter.next();
        if value.is_none() {
          return Err(Error::MissingArgumentValue(name.to_owned()));
        }
        value.unwrap()
      }
    };
    let payload = if is_valid(value) {
      MessageTransport::Success(Success::Json(value.to_owned()))
    } else {
      debug!(
        "Input '{}' for argument '{}' is not valid JSON. Wrapping it with quotes to make it a valid string value.",
        value, name
      );
      MessageTransport::Success(Success::Json(format!("\"{}\"", value)))
    };
    map.insert(name, payload);
  }

  Ok(map)
}

#[must_use]
fn split_arg(arg: &str) -> (&str, Option<&str>) {
  let mut parts = arg.split('=');
  (parts.next().unwrap(), parts.next())
}

fn is_valid(string: &str) -> bool {
  let parsed: Result<serde_json::Value, _> = serde_json::from_str(string);
  parsed.is_ok()
}

#[cfg(test)]
mod tests {
  use anyhow::Result;

  use super::*;

  fn to_vec(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
  }

  #[test_logger::test]
  fn parse_separate_args() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a"]);
    let mut map = parse_args(&args)?;
    let value: String = map.consume("input-a")?;
    assert_eq!(value, "value-a");
    Ok(())
  }

  #[test_logger::test]
  fn parse_combined_args() -> Result<()> {
    let args = to_vec(&["--input-a=value-a"]);
    let mut map = parse_args(&args)?;
    let value: String = map.consume("input-a")?;
    assert_eq!(value, "value-a");
    Ok(())
  }

  #[test_logger::test]
  fn parse_mixed_args() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a", "--input-b=value-b"]);
    let mut map = parse_args(&args)?;
    let value: String = map.consume("input-a")?;
    assert_eq!(value, "value-a");
    let value: String = map.consume("input-b")?;
    assert_eq!(value, "value-b");
    Ok(())
  }

  #[test_logger::test]
  fn parse_err_invalid() -> Result<()> {
    let args = to_vec(&["input-a", "value-a", "--input-b=value-b"]);
    let result = parse_args(&args);
    assert!(result.is_err());
    Ok(())
  }

  #[test_logger::test]
  fn parse_err_dangling() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a", "--input-b"]);
    let result = parse_args(&args);
    assert!(result.is_err());
    Ok(())
  }

  #[test_logger::test]
  fn parse_arg_numeric() -> Result<()> {
    let args = to_vec(&["--num", "2000"]);
    let mut map = parse_args(&args)?;
    let value: i32 = map.consume("num")?;
    assert_eq!(value, 2000);
    Ok(())
  }

  #[test_logger::test]
  fn test_is_valid() -> Result<()> {
    let int = "1234567890";
    assert!(is_valid(int));
    let float = "12345.67890";
    assert!(is_valid(float));
    let obj = "{}";
    assert!(is_valid(obj));
    let array = "[]";
    assert!(is_valid(array));
    let naked_string = "hello world";
    assert!(!is_valid(naked_string));
    let string = "\"hello world\"";
    assert!(is_valid(string));
    Ok(())
  }
}
