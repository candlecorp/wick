use wick_packet::Packet;

use crate::Error;

/// Parse CLI arguments into a [wick_packet::PacketStream]
pub fn parse_args(args: &[String]) -> Result<Vec<Packet>, Error> {
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
    let payload = if is_valid(value) {
      Packet::encode(name, serde_json::from_str::<serde_json::Value>(value).unwrap())
    } else {
      debug!(
        "Input '{}' for argument '{}' is not valid JSON. Wrapping it with quotes to make it a valid string value.",
        value, name
      );
      Packet::encode(
        name,
        serde_json::from_str::<serde_json::Value>(&format!("\"{}\"", value)).unwrap(),
      )
    };
    packets.push(payload);
  }

  Ok(packets)
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
    let packets = parse_args(&args)?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    Ok(())
  }

  #[test_logger::test]
  fn parse_numbers() -> Result<()> {
    let args = to_vec(&["--input-a", "123"]);
    let packets = parse_args(&args)?;
    assert_eq!(packets[0], Packet::encode("input-a", 123));
    assert_eq!(packets[0].clone().decode::<i32>().unwrap(), 123);
    Ok(())
  }

  #[test_logger::test]
  fn parse_combined_args() -> Result<()> {
    let args = to_vec(&["--input-a=value-a"]);
    let packets = parse_args(&args)?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    Ok(())
  }

  #[test_logger::test]
  fn parse_mixed_args() -> Result<()> {
    let args = to_vec(&["--input-a", "value-a", "--input-b=value-b"]);
    let packets = parse_args(&args)?;
    assert_eq!(packets[0], Packet::encode("input-a", "value-a"));
    assert_eq!(packets[1], Packet::encode("input-b", "value-b"));
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
    let packets = parse_args(&args)?;
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
