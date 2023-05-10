#![allow(unused_attributes, clippy::box_default)]

mod test;

use anyhow::Result;
use pretty_assertions::assert_eq;
use serde_json::json;
use wick_packet::{packets, Packet};

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  first_packet_test("./tests/manifests/v0/core/senders.yaml", Vec::new(), "Hello world").await
}

#[test_logger::test(tokio::test)]
async fn test_pluck() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-pluck.yaml",
    packets!(("input", json!({ "to_pluck" :"Hello world!", "to_ignore": "ignore me" }))),
    "Hello world!",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_pluck_shorthand() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-pluck-shorthand.yaml",
    packets!(("input", json!({ "to_pluck" :"Hello world!", "to_ignore": "ignore me" }))),
    "Hello world!",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_drop() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-drop.yaml",
    packets!(("first", "first"), ("second", "second"), ("third", "third")),
    "second",
  )
  .await
}

#[test_logger::test(tokio::test)]
// #[ignore]
async fn test_merge() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v1/core-merge.yaml",
    "test",
    packets!(
      ("input_a", "first_value"),
      ("input_b", 2u8),
      ("input_c", ["alpha", "beta"])
    ),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual = wrapper.deserialize_generic()?;
  let expected = json!({"a": "first_value", "b": 2, "c": ["alpha", "beta"]});
  assert_eq!(actual, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
// #[ignore]
async fn test_sender_merge() -> Result<()> {
  let (interpreter, mut outputs) =
    test::common_setup("./tests/manifests/v1/core-sender-merge.yaml", "test", Vec::new()).await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual = wrapper.deserialize_generic()?;
  let expected = json!({"a": true, "b": "Hello world", "c": 123456});
  assert_eq!(actual, expected);
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
// #[ignore]
async fn test_multi_sender() -> Result<()> {
  let (interpreter, mut outputs) =
    test::common_setup("./tests/manifests/v1/core-multi-sender.yaml", "test", Vec::new()).await?;

  assert_eq!(outputs.len(), 6);

  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();

  assert_eq!(actual, Packet::encode("c", 123456));
  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();
  assert_eq!(actual, Packet::encode("b", "Hello world"));
  let _ = outputs.pop();
  let actual = outputs.pop().unwrap().unwrap();
  assert_eq!(actual, Packet::encode("a", true));
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_subflows() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/private-flows.yaml",
    packets!(("input", "hello WORLD")),
    "DLROW OLLEH",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_1() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch.yaml",
    packets!(("command", "want_reverse"), ("input", "hello WORLD")),
    "DLROW olleh",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_case_2() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch.yaml",
    packets!(("command", "want_uppercase"), ("input", "hello WORLD")),
    "HELLO WORLD",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_default() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch.yaml",
    packets!(("command", "nomatch"), ("input", "hello WORLD")),
    "hello WORLD",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_bool_true() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch-2.yaml",
    packets!(("input", true), ("message", "does not matter")),
    "on_true",
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn test_switch_bool_default() -> Result<()> {
  first_packet_test(
    "./tests/manifests/v1/core-switch-2.yaml",
    packets!(("input", false), ("message", "does not matter")),
    "on_false",
  )
  .await
}

async fn first_packet_test(file: &str, packets: Vec<Packet>, expected: &str) -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(file, "test", packets).await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual: String = wrapper.deserialize()?;
  assert_eq!(actual, expected);
  interpreter.shutdown().await?;

  Ok(())
}

// #[test_logger::test(tokio::test)]
// async fn test_switch_err() -> Result<()> {
//   error_test(
//     "./tests/manifests/v1/core-switch.yaml",
//     Packet::err("match", "Something bad"),
//     "hello WORLD",
//   )
//   .await
// }
// async fn error_test(file: &str, packet: Packet, expected: &str) -> Result<()> {
//   let (interpreter, mut outputs) = test::common_setup(file, "test", vec![packet]).await?;

//   assert_eq!(outputs.len(), 2);

//   let _ = outputs.pop();
//   let wrapper = outputs.pop().unwrap().unwrap();
//   let actual: String = wrapper.deserialize()?;
//   assert_eq!(actual, expected);
//   interpreter.shutdown().await?;

//   Ok(())
// }
// #[test_logger::test(tokio::test)]
// async fn test_merge() -> Result<()> {
//   let manifest = load("./tests/manifests/v0/core/merge.yaml")?;
//   let network = from_def(&manifest)?;
//   let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);
//   let mut inputs = PacketMap::default();
//   inputs.insert("schem_one", "first value");
//   inputs.insert("schem_two", 2u8);
//   inputs.insert("schem_three", &["alpha".to_owned(), "beta".to_owned()]);

//   let invocation = Invocation::new_test("merge", Entity::local("test"), inputs, None);
//   let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
//   interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
//   let mut stream = interpreter.invoke(invocation).await?;

//   let mut outputs: Vec<_> = stream.drain().await;
//   println!("{:#?}", outputs);

//   let wrapper = outputs.pop().unwrap();

//   #[derive(serde::Deserialize, PartialEq, Debug)]
//   struct Merged {
//     one: String,
//     two: i32,
//     three: Vec<String>,
//   }

//   let result: Merged = wrapper.deserialize()?;

//   assert_eq!(
//     result,
//     Merged {
//       one: "first value".to_owned(),
//       two: 2,
//       three: vec!["alpha".to_owned(), "beta".to_owned()]
//     }
//   );
//   interpreter.shutdown().await?;

//   Ok(())
// }
