use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::future::BoxFuture;
use seeded_random::{Random, Seed};
use serde_json::Value;
use tokio::task::JoinHandle;
use tracing::trace;
use wasmflow_interpreter::{BoxError, Collection};
use wasmflow_sdk::v1::transport::{Failure, MessageTransport, TransportStream, TransportWrapper};
use wasmflow_sdk::v1::types::{CollectionFeatures, CollectionSignature, ComponentSignature, TypeSignature};
use wasmflow_sdk::v1::{CollectionLink, Invocation};

pub struct TestCollection(CollectionSignature);
impl TestCollection {
  #[allow(dead_code)]
  pub fn new() -> Self {
    let signature = CollectionSignature::new("test-collection")
      .format(1)
      .version("0.0.0")
      .features(CollectionFeatures::v0(false, false))
      .add_component(
        ComponentSignature::new("echo")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("timeout")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("timeout-nodone")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("concat")
          .add_input("left", TypeSignature::String)
          .add_input("right", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("concat-five")
          .add_input("one", TypeSignature::String)
          .add_input("two", TypeSignature::String)
          .add_input("three", TypeSignature::String)
          .add_input("four", TypeSignature::String)
          .add_input("five", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("splitter")
          .add_input("input", TypeSignature::String)
          .add_output("rest", TypeSignature::String)
          .add_output("vowels", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("ref_to_string")
          .add_input("link", TypeSignature::Link { schemas: vec![] })
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("exception")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("panic")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("copy")
          .add_input("input", TypeSignature::String)
          .add_input("times", TypeSignature::U64)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("reverse")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_component(
        ComponentSignature::new("render")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      );

    Self(signature)
  }
}

type Sender = Box<dyn FnMut(TransportWrapper) -> JoinHandle<()> + Send + Sync>;

fn stream(_seed: u64) -> (Sender, TransportStream) {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  let tx = Arc::new(tx);

  let stream = TransportStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
  let mut total = 0;
  let sender: Sender = Box::new(move |msg: TransportWrapper| {
    let rng = Random::new();
    let delay = rng.range(10, 200);
    total += delay;
    let tx = tx.clone();
    tokio::spawn(async move {
      trace!(total, "sleeping for delayed send");
      tokio::time::sleep(Duration::from_millis(total.into())).await;
      let _ = tx.send(msg);
    })
  });
  (sender, stream)
}

fn defer(futs: Vec<JoinHandle<()>>) {
  tokio::spawn(async move {
    let rng = Random::from_seed(Seed::unsafe_new(1));
    let millis = rng.range(10, 100);
    trace!(millis, "sleeping");
    tokio::time::sleep(Duration::from_millis(millis.into())).await;
    for fut in futs {
      fut.await.unwrap();
    }
  });
}

impl Collection for TestCollection {
  fn handle(&self, mut invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    let operation = invocation.target.name();
    println!("got op {} in echo test collection", operation);
    let stream = match operation {
      "echo" => {
        let input = invocation.payload.remove("input").unwrap();

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", input)),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "concat" => {
        let left: String = invocation.payload.consume("left").unwrap();
        let right: String = invocation.payload.consume("right").unwrap();
        let result = format!("{}{}", left, right);

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&result))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "concat-five" => {
        let one: String = invocation.payload.consume("one").unwrap();
        let two: String = invocation.payload.consume("two").unwrap();
        let three: String = invocation.payload.consume("three").unwrap();
        let four: String = invocation.payload.consume("four").unwrap();
        let five: String = invocation.payload.consume("five").unwrap();
        let result = format!("{}{}{}{}{}", one, two, three, four, five);

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&result))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "timeout" => {
        let input: String = invocation.payload.consume("input").unwrap();

        // tx.send(TransportWrapper::new("output", MessageTransport::success(&input)))
        //   .unwrap();
        // tx.send(TransportWrapper::done("output")).unwrap();

        // // keep the sender hanging around until passed the timeout.
        // tokio::spawn(async move {
        //   tokio::time::sleep(Duration::from_millis(2000)).await;
        //   if tx.is_closed() {
        //     println!("tx closed");
        //   }
        // });

        // let stream = TransportStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
        // Ok(stream)

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&input))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "timeout-nodone" => {
        let input: String = invocation.payload.consume("input").unwrap();

        // let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // // Don't send "done"
        // tx.send(TransportWrapper::new("output", MessageTransport::success(&input)))
        //   .unwrap();

        // // keep the sender hanging around until passed the timeout.
        // tokio::spawn(async move {
        //   tokio::time::sleep(Duration::from_millis(2000)).await;
        //   if tx.is_closed() {
        //     println!("tx closed");
        //   }
        // });

        // let stream = TransportStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
        // Ok(stream)

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&input))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "splitter" => {
        let input: String = invocation.payload.consume("input").unwrap();
        let vowels: Vec<_> = input
          .chars()
          .filter(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
          .collect();
        let rest: Vec<_> = input
          .chars()
          .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
          .collect();

        let (mut send, stream) = stream(1);

        // send all the vowels immediately.
        let mut futs = vowels
          .iter()
          .map(|v| send(TransportWrapper::new("vowels", MessageTransport::success(v))))
          .collect::<Vec<_>>();
        futs.push(send(TransportWrapper::done("vowels")));
        defer(futs);

        let mut futs = rest
          .iter()
          .map(|v| send(TransportWrapper::new("rest", MessageTransport::success(v))))
          .collect::<Vec<_>>();
        futs.push(send(TransportWrapper::done("rest")));
        defer(futs);

        // for vowel in vowels {
        //   tx.send().unwrap();
        // }
        // tx.send(TransportWrapper::done("vowels")).unwrap();

        // send consonants asynchronously with a delay.
        // tokio::spawn(async move {
        //   for other in rest {
        //     tokio::time::sleep(Duration::from_millis(200)).await;
        //     tx.send(TransportWrapper::new("rest", MessageTransport::success(&other)))
        //       .unwrap();
        //   }
        //   tx.send(TransportWrapper::done("rest")).unwrap();
        // });

        Ok(stream)
      }
      "ref_to_string" => {
        let link: CollectionLink = invocation.payload.consume("link").unwrap();
        let result = link.to_string();

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&result))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "reverse" => {
        println!("Reverse payload {:?}", invocation.payload);
        let input: String = invocation.payload.consume("input").unwrap();

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new(
            "output",
            MessageTransport::success(&input.chars().rev().collect::<String>()),
          )),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "copy" => {
        println!("Reverse payload {:?}", invocation.payload);
        let input: String = invocation.payload.consume("input").unwrap();
        let times: u64 = invocation.payload.consume("times").unwrap();
        let mut futs = vec![];

        let (mut send, stream) = stream(1);
        for _ in 0..times {
          futs.push(send(TransportWrapper::new("output", MessageTransport::success(&input))));
        }

        futs.push(send(TransportWrapper::done("output")));

        defer(futs);

        Ok(stream)
      }
      "exception" => {
        let input = invocation.payload.remove("input").unwrap();
        println!("test::exception got {}", input);

        let (mut send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new(
            "output",
            MessageTransport::Failure(Failure::Exception("test::exception".to_owned())),
          )),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "panic" => {
        let input = invocation.payload.remove("input").unwrap();
        println!("test::panic got {}", input);
        panic!();
      }
      _ => Err("Error".into()),
    };
    Box::pin(async move { stream })
  }

  fn list(&self) -> &CollectionSignature {
    &self.0
  }
}
