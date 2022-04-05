use std::path::Path;
use std::time::SystemTime;

mod test;

use anyhow::Result;
use test::{JsonWriter, TestProvider};
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_interpreter::graph::from_def;
use vino_interpreter::{HandlerMap, Interpreter, InterpreterOptions, ProviderNamespace};
use vino_manifest::Loadable;
use vino_random::Seed;
use vino_transport::{Invocation, MessageTransport, TransportMap};

fn load<T: AsRef<Path>>(path: T) -> Result<vino_manifest::HostManifest> {
  Ok(vino_manifest::HostManifest::load_from_file(path.as_ref())?)
}

const OPTIONS: Option<InterpreterOptions> = Some(InterpreterOptions {
  error_on_hung: false,
  error_on_missing: true,
});

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/panic.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("panic", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_done_noclose() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-done-noclose.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;

  let start = SystemTime::now();

  let stream = interpreter.invoke(invocation).await?;
  let mut outputs: Vec<_> = stream.collect().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 2);

  let elapsed = start.elapsed()?;
  println!("Elapsed: {:?} ", elapsed);

  // // Assert that we've taken no longer than the HUNG_TX_TIMEOUT (within a window)
  // let buffer = Duration::from_millis(500);
  // assert!(elapsed + buffer > EventLoop::HUNG_TX_TIMEOUT * 2);
  // assert!(elapsed - buffer < EventLoop::HUNG_TX_TIMEOUT * 2);

  assert_eq!(outputs.len(), 2);
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Signal(_)));
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_timeout_missingdone() -> Result<()> {
  let manifest = load("./tests/manifests/v0/bad-cases/timeout-missingdone.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = HandlerMap::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "hello world".to_owned())]);
  let invocation = Invocation::new_test("timeout-nodone", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers))?;
  interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;

  let start = SystemTime::now();

  let stream = interpreter.invoke(invocation).await?;
  let mut outputs: Vec<_> = stream.collect().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 2);
  let elapsed = start.elapsed()?;
  println!("Elapsed: {:?} ", elapsed);

  // // Assert that we've taken no longer than the HUNG_TX_TIMEOUT (within a window)
  // let buffer = Duration::from_millis(500);
  // assert!(elapsed + buffer > EventLoop::HUNG_TX_TIMEOUT * 2);
  // assert!(elapsed - buffer < EventLoop::HUNG_TX_TIMEOUT * 2);

  assert_eq!(outputs.len(), 2);
  let wrapper = outputs.pop().unwrap();
  // TODO: need to make this a failure.
  assert!(matches!(wrapper.payload, MessageTransport::Signal(_)));
  // assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}
