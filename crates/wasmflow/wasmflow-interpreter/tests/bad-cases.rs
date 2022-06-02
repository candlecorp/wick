use std::path::Path;
use std::time::SystemTime;

mod test;

use anyhow::Result;
use test::{JsonWriter, TestProvider};
use wasmflow_interpreter::graph::from_def;
use wasmflow_interpreter::{HandlerMap, Interpreter, InterpreterOptions, NamespaceHandler};
use wasmflow_manifest::Loadable;
use seeded_random::Seed;
use wasmflow_transport::MessageTransport;
use wasmflow_entity::Entity;
use wasmflow_invocation::Invocation;
use wasmflow_packet::PacketMap;

fn load<T: AsRef<Path>>(path: T) -> Result<wasmflow_manifest::HostManifest> {
  Ok(wasmflow_manifest::HostManifest::load_from_file(path.as_ref())?)
}

const OPTIONS: Option<InterpreterOptions> = Some(InterpreterOptions {
  error_on_hung: false,
  error_on_missing: true,
});

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/panic.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestProvider::new()))]);
  let inputs = PacketMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("panic", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
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
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-done-noclose.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestProvider::new()))]);

  let inputs = PacketMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
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
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-missingdone.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestProvider::new()))]);

  let inputs = PacketMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout-nodone", Entity::local("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
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
