use std::borrow::Cow;

use vino_manifest::default::process_default;
use vino_transport::MessageTransport;

use crate::dev::prelude::*;

pub(crate) fn make_default_transport(json: &serde_json::Value, message: &str) -> MessageTransport {
  process_default(Cow::Borrowed(json), message).map_or(
    MessageTransport::Error("Error processing default value".to_owned()),
    |result| {
      mp_serialize(&result).map_or(
        MessageTransport::Error("Error serializing default value".to_owned()),
        MessageTransport::MessagePack,
      )
    },
  )
}

#[cfg(test)]
mod tests {

  use vino_manifest::default::parse_default;

  use super::*;
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  #[test_env_log::test]
  fn test_to_transport() -> TestResult<()> {
    let json_str = r#"
    "Error: $ERROR"
    "#;

    let json = parse_default(json_str)?;

    let err = "This is my error message";
    let message: String = make_default_transport(&json, err).try_into()?;

    assert_eq!(message, format!("Error: {}", err));

    Ok(())
  }
}
