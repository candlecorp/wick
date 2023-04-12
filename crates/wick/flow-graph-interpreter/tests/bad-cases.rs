#![allow(unused_attributes, clippy::box_default)]

mod test;
use anyhow::Result;
use pretty_assertions::assert_eq;
use wick_packet::{packets, Packet};

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/bad-cases/panic.yaml",
    "test",
    packets!(("input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  outputs.pop();
  let p = outputs.pop().unwrap().unwrap();
  assert_eq!(p, Packet::err("output", "Operation wick://test/panic panicked"));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_error() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/bad-cases/error.yaml",
    "test",
    packets!(("input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  outputs.pop();
  let p = outputs.pop().unwrap().unwrap();
  assert_eq!(
    p,
    Packet::err(
      "output",
      "Operation wick://test/error failed: Component error: This operation always errors"
    )
  );

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
// #[ignore]
async fn test_timeout_done_noclose() -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(
    "./tests/manifests/v0/bad-cases/timeout-done-noclose.yaml",
    "test",
    packets!(("input", "Hello world")),
  )
  .await?;

  assert_eq!(outputs.len(), 2);

  outputs.pop();
  let p = outputs.pop().unwrap().unwrap();
  assert_eq!(p, Packet::err("output", "component failed to produce output"));

  interpreter.shutdown().await?;

  Ok(())
}

// #[test_logger::test(tokio::test)]
// async fn test_timeout_missingdone() -> Result<()> {
//   let manifest = load("./tests/manifests/v0/bad-cases/timeout-missingdone.yaml")?;
//   let network = from_def(&manifest)?;
//   let collections = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(TestCollection::new()))]);

//   let inputs = PacketMap::from([("input", "hello world".to_owned())]);
//   let invocation = Invocation::new_test("timeout-nodone", Entity::local("test"), inputs, None);
//   let mut interpreter = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections))?;
//   interpreter.start(OPTIONS, Some(Box::new(JsonWriter::default()))).await;

//   let start = SystemTime::now();

//   let mut stream = interpreter.invoke(invocation).await?;
//   let mut outputs: Vec<_> = stream.drain().await;
//   interpreter.shutdown().await?;
//   println!("{:#?}", outputs);
//   assert_eq!(outputs.len(), 1);
//   let elapsed = start.elapsed()?;
//   println!("Elapsed: {:?} ", elapsed);

//   // assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));
//   let wrapper = outputs.pop().unwrap();
//   assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

//   Ok(())
// }
