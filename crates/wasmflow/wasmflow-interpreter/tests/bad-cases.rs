#![allow(unused_attributes, clippy::box_default)]

use std::path::Path;
use std::time::SystemTime;

mod test;

use anyhow::Result;
use seeded_random::Seed;
use test::{JsonWriter, TestCollection};
use wasmflow_interpreter::graph::from_def;
use wasmflow_interpreter::{HandlerMap, Interpreter, InterpreterOptions, NamespaceHandler};
use wasmflow_sdk::v1::packet::PacketMap;
use wasmflow_sdk::v1::transport::MessageTransport;
use wasmflow_sdk::v1::{Entity, Invocation};

fn load<T: AsRef<Path>>(path: T) -> Result<wasmflow_manifest::WasmflowManifest> {
  Ok(wasmflow_manifest::WasmflowManifest::load_from_file(path.as_ref())?)
}

const OPTIONS: Option<InterpreterOptions> = Some(InterpreterOptions {
  error_on_hung: false,
  error_on_missing: true,
});

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/panic.wafl")?;
  let network = from_def(&manifest)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);
  let inputs = PacketMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("panic", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
  let mut stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.drain().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_done_noclose() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-done-noclose.wafl")?;
  let network = from_def(&manifest)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);

  let inputs = PacketMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;

  let start = SystemTime::now();

  let mut stream = interpreter.invoke(invocation).await?;
  let mut outputs: Vec<_> = stream.drain().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);

  let elapsed = start.elapsed()?;
  println!("Elapsed: {:?} ", elapsed);

  assert_eq!(outputs.len(), 1);

  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_missingdone() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-missingdone.wafl")?;
  let network = from_def(&manifest)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);

  let inputs = PacketMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout-nodone", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;

  let start = SystemTime::now();

  let mut stream = interpreter.invoke(invocation).await?;
  let mut outputs: Vec<_> = stream.drain().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 1);
  let elapsed = start.elapsed()?;
  println!("Elapsed: {:?} ", elapsed);

  // assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}
