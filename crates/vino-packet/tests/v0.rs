use anyhow::Result;
use log::debug;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_packet::v0::Payload;
use vino_packet::Packet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructOne {
  one: i32,
  two: String,
  nested: StructTwo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructTwo {
  one: i32,
  two: String,
}

/*********************************************************************
  Known values used for backwards compatibility testing. Don't change.
*********************************************************************/

// Serializable("Hello world") (should turn into MessageBack(bytes))
static STRING_BYTES: [u8; 20] = [
  129, 161, 48, 129, 161, 51, 156, 204, 171, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
];
// Exception("Test exception message")
static EXCEPTION_BYTES: [u8; 29] = [
  129, 161, 48, 129, 161, 49, 182, 84, 101, 115, 116, 32, 101, 120, 99, 101, 112, 116, 105, 111,
  110, 32, 109, 101, 115, 115, 97, 103, 101,
];
// Exception("Test error message")
static ERROR_BYTES: [u8; 25] = [
  129, 161, 48, 129, 161, 50, 178, 84, 101, 115, 116, 32, 101, 114, 114, 111, 114, 32, 109, 101,
  115, 115, 97, 103, 101,
];

#[test_env_log::test]
fn serializable() -> Result<()> {
  let user_data = "Hello world";
  let user_bytes = serialize(&user_data)?;
  let output2 = Packet::V0(Payload::MessagePack(user_bytes.clone()));
  debug!("test serializable()");
  debug!("messagepack: {:?}", output2);
  let bytes2 = serialize(&output2)?;
  debug!("messagepack bytes: {:?}", bytes2);
  let payload: Packet = deserialize(&STRING_BYTES)?;
  assert_eq!(payload, Packet::V0(Payload::MessagePack(user_bytes)));
  assert_eq!(bytes2, STRING_BYTES.to_vec());
  Ok(())
}

#[test_env_log::test]
fn exception() -> Result<()> {
  let user_data = "Test exception message";
  let output = Packet::V0(Payload::Exception(user_data.to_string()));
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  assert_eq!(bytes, EXCEPTION_BYTES.to_vec());
  let payload: Packet = deserialize(&EXCEPTION_BYTES)?;
  assert_eq!(
    payload,
    Packet::V0(Payload::Exception(user_data.to_string()))
  );
  Ok(())
}

#[test_env_log::test]
fn error() -> Result<()> {
  let user_data = "Test error message";
  let output = Packet::V0(Payload::Error(user_data.to_string()));
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  assert_eq!(bytes, ERROR_BYTES.to_vec());
  let payload: Packet = deserialize(&ERROR_BYTES)?;
  assert_eq!(payload, Packet::V0(Payload::Error(user_data.to_string())));
  Ok(())
}

#[test_env_log::test]
fn invalid() -> Result<()> {
  let output = Packet::V0(Payload::Invalid);
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  // assert_eq!(bytes, ERROR_BYTES.to_vec());
  let payload: Packet = deserialize(&bytes)?;
  assert_eq!(payload, Packet::V0(Payload::Invalid));
  Ok(())
}

#[test_env_log::test]
fn basic_msgpack() -> Result<()> {
  let user_data = "Test error message";
  let user_bytes = serialize(&user_data)?;
  debug!("user bytes: {:?}", user_bytes);
  let output = Packet::V0(Payload::MessagePack(user_bytes.clone()));
  debug!("output: {:?}", output);
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  // assert_eq!(bytes, ERROR_BYTES.to_vec());
  let payload: Packet = deserialize(&bytes)?;
  debug!("msgpack deserialized: {:?}", payload);
  assert_eq!(payload, Packet::V0(Payload::MessagePack(user_bytes)));
  Ok(())
}

#[test_env_log::test]
fn msgpack_struct() -> Result<()> {
  let user_data = StructOne {
    one: 1,
    two: "hello world".to_string(),
    nested: StructTwo {
      one: 32232,
      two: "nested struct".to_string(),
    },
  };
  let user_bytes = serialize(&user_data)?;
  debug!("user bytes: {:?}", user_bytes);
  let output = Packet::V0(Payload::MessagePack(user_bytes.clone()));
  debug!("output: {:?}", output);
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  // assert_eq!(bytes, ERROR_BYTES.to_vec());
  let payload: Packet = deserialize(&bytes)?;
  debug!("msgpack deserialized: {:?}", payload);
  assert_eq!(payload, Packet::V0(Payload::MessagePack(user_bytes)));
  Ok(())
}

#[test_env_log::test]
fn deserialize_unknown() -> Result<()> {
  let user_data = StructOne {
    one: 1,
    two: "hello world".to_string(),
    nested: StructTwo {
      one: 32232,
      two: "nested struct".to_string(),
    },
  };
  let user_bytes = serialize(&user_data)?;
  debug!("user bytes: {:?}", user_bytes);
  let output = Packet::V0(Payload::MessagePack(user_bytes.clone()));
  debug!("output: {:?}", output);
  let bytes = serialize(&output)?;
  debug!("bytes: {:?}", bytes);
  // assert_eq!(bytes, ERROR_BYTES.to_vec());
  let payload: Packet = deserialize(&bytes)?;
  debug!("msgpack deserialized: {:?}", payload);
  assert_eq!(payload, Packet::V0(Payload::MessagePack(user_bytes)));
  Ok(())
}
