#![allow(unused_attributes, clippy::box_default)]
use std::path::Path;

mod test;

use anyhow::Result;
use seeded_random::Seed;
use test::{JsonWriter, TestCollection};
use wasmflow_interpreter::graph::from_def;
use wasmflow_interpreter::{HandlerMap, Interpreter, InterpreterOptions, NamespaceHandler};
use wasmflow_sdk::v1::packet::PacketMap;
use wasmflow_sdk::v1::{Entity, Invocation};

fn load<T: AsRef<Path>>(path: T) -> Result<wasmflow_manifest::WasmflowManifest> {
  Ok(wasmflow_manifest::WasmflowManifest::load_from_file(path.as_ref())?)
}

const OPTIONS: Option<InterpreterOptions> = Some(InterpreterOptions {
  error_on_hung: true,
  // TODO: improve logic to ensure no remaining packets are sent after completion.
  // Turn this on to make tests fail in these cases.
  error_on_missing: false,
});

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  let manifest = load("./tests/manifests/v0/core/senders.wafl")?;
  let network = from_def(&manifest)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);
  let inputs = PacketMap::default();

  let invocation = Invocation::new_test("senders", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
  let mut stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.drain().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  let result: String = wrapper.deserialize()?;

  assert_eq!(result, "Hello world".to_owned());
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_merge() -> Result<()> {
  let manifest = load("./tests/manifests/v0/core/merge.wafl")?;
  let network = from_def(&manifest)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);
  let mut inputs = PacketMap::default();
  inputs.insert("schem_one", "first value");
  inputs.insert("schem_two", 2u8);
  inputs.insert("schem_three", &["alpha".to_owned(), "beta".to_owned()]);

  let invocation = Invocation::new_test("merge", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
  let mut stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.drain().await;
  println!("{:#?}", outputs);

  let wrapper = outputs.pop().unwrap();

  #[derive(serde::Deserialize, PartialEq, Debug)]
  struct Merged {
    one: String,
    two: i32,
    three: Vec<String>,
  }

  let result: Merged = wrapper.deserialize()?;

  assert_eq!(
    result,
    Merged {
      one: "first value".to_owned(),
      two: 2,
      three: vec!["alpha".to_owned(), "beta".to_owned()]
    }
  );
  interpreter.shutdown().await?;

  Ok(())
}
