use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::{
  json,
  messagepack,
  raw,
  Error,
};

#[test]
pub fn json_ser() -> Result<(), Error> {
  #[derive(Serialize, Deserialize)]
  struct Point {
    x: i32,
    y: i32,
  }

  let point = Point { x: 200, y: 193 };

  let value = json::serialize(&point)?;
  println!("{:?}", value);

  assert_eq!(value, r#"{"x":200,"y":193}"#);
  Ok(())
}

#[test]
pub fn json_de() -> Result<(), Error> {
  #[derive(Serialize, Deserialize, Debug, PartialEq)]
  struct Point {
    x: i32,
    y: i32,
  }

  let json = r#"{"x":200,"y":193}"#;

  let instance: Point = json::deserialize(json)?;

  assert_eq!(instance, Point { x: 200, y: 193 });
  Ok(())
}

#[test]
pub fn mp_ser() -> Result<(), Error> {
  #[derive(Serialize, Deserialize)]
  struct Point {
    x: i32,
    y: i32,
  }

  let point = Point { x: 200, y: 193 };

  let value = messagepack::serialize(&point)?;
  println!("{:?}", value);

  let expected: Vec<u8> = vec![146, 204, 200, 204, 193];
  assert_eq!(value, expected);
  Ok(())
}

#[test]
pub fn mp_de() -> Result<(), Error> {
  #[derive(Serialize, Deserialize, Debug, PartialEq)]
  struct Point {
    x: i32,
    y: i32,
  }

  let slice = vec![146, 204, 200, 204, 193];

  let instance: Point = messagepack::deserialize(&slice)?;

  assert_eq!(instance, Point { x: 200, y: 193 });
  Ok(())
}

#[test]
pub fn raw_rt() -> Result<(), Error> {
  #[derive(Serialize, Deserialize, Debug, PartialEq)]
  struct Point {
    x: i32,
    y: i32,
  }

  let point = Point { x: 200, y: 193 };

  let value = raw::serialize(&point)?;
  let instance: Point = raw::deserialize(value)?;

  assert_eq!(instance, Point { x: 200, y: 193 });
  Ok(())
}
