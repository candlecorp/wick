#![allow(unused_attributes, clippy::box_default)]

mod test;

use anyhow::Result;
use flow_graph_interpreter::graph::from_def;
use rot::assert_equal;
use seeded_random::Seed;
use wick_packet::Packet;

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  let (interpreter, mut outputs) = interp!("./tests/manifests/v0/core/senders.yaml", "test", Vec::new());

  assert_equal!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let expected = Packet::encode("output", "Hello world");
  assert_equal!(wrapper, expected);
  interpreter.shutdown().await?;

  Ok(())
}

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
