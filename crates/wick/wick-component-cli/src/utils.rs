use serde::Deserialize;
use serde_json::Value;
use wick_interface_types::{OperationSignature, Type};
use wick_packet::Packet;

use crate::Error;

/// Parse CLI arguments into a [wick_packet::PacketStream]
pub fn parse_args(args: &[String], sig: &OperationSignature) -> Result<Vec<Packet>, Error> {
  let mut packets = Vec::new();
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
    let input = sig.inputs.iter().find(|i| i.name == name);
    if input.is_none() {
      return Err(Error::InvalidInput(name.to_owned()));
    }
    let input = input.unwrap();
    let value: Value = match input.ty() {
      // Datetime can be parsed from a string or a number but number's need to be stringified.
      // Strings must be explicit because a bare number will be parsed as a number.
      Type::Datetime | Type::String => {
        if is_valid(value) {
          coerce_string(name, value, input.ty())?
        } else {
          // if it's not valid JSON then it's a bare string.
          value.into()
        }
      }
      // serde_json does an adequate job on the rest.
      _ => encode::<Value>(name, value, input.ty())?,
    };
    // Note on above: complex objects with embedded Datetime/Strings
    // may not be parsed correctly but that's an edge case we're ignoring for now.

    let payload = Packet::encode(name, value);

    packets.push(payload);
  }

  Ok(packets)
}

fn encode<'de, T: Deserialize<'de>>(name: &str, value: &'de str, ty: &Type) -> Result<T, Error> {
  serde_json::from_str(value).map_err(|_e| Error::Encoding(name.to_owned(), value.to_owned(), ty.clone()))
}

fn coerce_string(name: &str, value: &str, ty: &Type) -> Result<Value, Error> {
  let val = serde_json::from_str::<Value>(value)
    .map_err(|_e| Error::Encoding(name.to_owned(), value.to_owned(), ty.clone()))?;
  Ok(match val {
    serde_json::Value::Null => Value::String("null".to_owned()),
    serde_json::Value::Bool(v) => Value::String(v.to_string()),
    serde_json::Value::Number(v) => Value::String(v.to_string()),
    serde_json::Value::String(v) => Value::String(v),
    serde_json::Value::Array(_v) => return Err(Error::Encoding(name.to_owned(), value.to_owned(), ty.clone())),
    serde_json::Value::Object(_v) => return Err(Error::Encoding(name.to_owned(), value.to_owned(), ty.clone())),
  })
}

#[must_use]
fn split_arg(arg: &str) -> (&str, Option<&str>) {
  let mut parts = arg.split('=');
  (parts.next().unwrap(), parts.next())
}

fn is_valid(string: &str) -> bool {
  let parsed: Result<Value, _> = serde_json::from_str(string);
  parsed.is_ok()
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use wick_interface_types::Field;
  use wick_packet::DateTime;

  use super::*;

  fn to_vec(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
  }

  fn sig(fields: &[(&str, Type)]) -> OperationSignature {
    OperationSignature {
      name: "test".to_owned(),
      config: Default::default(),
      inputs: fields.iter().map(|(n, t)| Field::new(n, t.clone())).collect(),
      outputs: vec![],
    }
  }

  #[test_logger::test]
  fn parse_separate_args() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a"]);
    let packets = parse_args(&args, &sig(&[("input-a", Type::String)]))?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    Ok(())
  }

  #[test_logger::test]
  fn parse_numbers() -> Result<()> {
    let args = to_vec(&["--input-a", "123"]);
    let packets = parse_args(&args, &sig(&[("input-a", Type::U64)]))?;
    assert_eq!(packets[0], Packet::encode("input-a", 123));
    assert_eq!(packets[0].clone().decode::<i32>().unwrap(), 123);
    Ok(())
  }

  #[test_logger::test]
  #[ignore = "This is broken and should be fixed."]
  fn parse_date_millis() -> Result<()> {
    let date = wick_packet::parse_date("2021-04-12T22:10:57+02:00")?;
    let args = to_vec(&["--input-a", &date.timestamp_millis().to_string()]);
    println!("args: {:?}", args);
    let packets = parse_args(&args, &sig(&[("input-a", Type::Datetime)]))?;
    assert_eq!(packets[0].clone().decode::<DateTime>().unwrap(), date);
    Ok(())
  }

  #[test_logger::test]
  fn parse_date_str() -> Result<()> {
    let date = wick_packet::parse_date("2021-04-12T22:10:57+02:00")?;
    let args = to_vec(&["--input-a", "2021-04-12T22:10:57+02:00"]);
    println!("args: {:?}", args);
    let packets = parse_args(&args, &sig(&[("input-a", Type::Datetime)]))?;
    assert_eq!(packets[0].clone().decode::<DateTime>().unwrap(), date);
    Ok(())
  }

  #[test_logger::test]
  fn parse_combined_args() -> Result<()> {
    let args = to_vec(&["--input-a=value-a"]);
    let packets = parse_args(&args, &sig(&[("input-a", Type::String)]))?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    Ok(())
  }

  #[test_logger::test]
  fn parse_mixed_args() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a", "--input-b=value-b"]);
    let packets = parse_args(&args, &sig(&[("input-a", Type::String), ("input-b", Type::String)]))?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    assert_eq!(packets[1], Packet::encode("input-b", "value-b"));
    Ok(())
  }

  #[test_logger::test]
  fn parse_err_invalid() -> Result<()> {
    let args = to_vec(&["input-a", "value-a", "--input-b=value-b"]);
    let result = parse_args(&args, &sig(&[("input-a", Type::String), ("input-b", Type::String)]));
    assert!(result.is_err());
    Ok(())
  }

  #[test_logger::test]
  fn parse_err_dangling() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a", "--input-b"]);
    let result = parse_args(&args, &sig(&[("input-a", Type::String), ("input-b", Type::String)]));
    assert!(result.is_err());
    Ok(())
  }

  #[test_logger::test]
  fn parse_arg_numeric() -> Result<()> {
    let args = to_vec(&["--num", "2000"]);
    let packets = parse_args(&args, &sig(&[("num", Type::U32)]))?;
    assert_eq!(packets[0], Packet::encode("num", 2000));
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
