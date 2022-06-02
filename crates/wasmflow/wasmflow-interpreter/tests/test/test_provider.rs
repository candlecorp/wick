use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use tokio::task::JoinHandle;
use tracing::trace;
use wasmflow_interpreter::{BoxError, Provider};
use seeded_random::{Random, Seed};
use wasmflow_transport::{Failure, MessageTransport, TransportStream, TransportWrapper};
use wasmflow_collection_link::ProviderLink;
use wasmflow_interface::ProviderSignature;

pub struct TestProvider(ProviderSignature);
impl TestProvider {
  pub fn new() -> Self {
    let sig = serde_json::from_value(json!({
      "name":"test-provider",
      "format":1,
      "version": "",
        "components" : {
          "echo": {
            "name": "echo",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "timeout": {
            "name": "timeout",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "timeout-nodone": {
            "name": "timeout-nodone",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "concat": {
            "name": "concat",
            "inputs": {
              "left": {"type":"string"},
              "right": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "concat-five": {
            "name": "concat-five",
            "inputs": {
              "one": {"type":"string"},
              "two": {"type":"string"},
              "three": {"type":"string"},
              "four": {"type":"string"},
              "five": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "splitter": {
            "name": "splitter",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "rest": {"type":"string"},
              "vowels": {"type":"string"},
            }
          },
          "ref_to_string": {
            "name": "ref_to_string",
            "inputs": {
              "link": {"type":"link"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "exception": {
            "name": "exception",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "panic": {
            "name": "panic",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "copy": {
            "name": "copy",
            "inputs": {
              "input": {"type":"string"},
              "times": {"type":"u64"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "reverse": {
            "name": "reverse",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "render": {
            "name": "render",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          }
        }
    }))
    .unwrap();
    Self(sig)
  }
}

type Sender = Box<dyn Fn(TransportWrapper) -> JoinHandle<()> + Send + Sync>;

fn stream(seed: u64) -> (Sender, TransportStream) {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  let tx = Arc::new(tx);

  let stream = TransportStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(rx));
  let sender: Sender = Box::new(move |msg: TransportWrapper| {
    let rng = Random::from_seed(Seed::unsafe_new(seed));
    let millis = rng.range(100, 300);
    let tx = tx.clone();
    tokio::spawn(async move {
      trace!(millis, "sleeping");
      tokio::time::sleep(Duration::from_millis(millis.into())).await;
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

impl Provider for TestProvider {
  fn handle(
    &self,
    mut invocation: wasmflow_invocation::Invocation,
    _config: Option<Value>,
  ) -> BoxFuture<Result<TransportStream, BoxError>> {
    let operation = invocation.target.name();
    println!("got op {} in echo test provider", operation);
    let stream = match operation {
      "echo" => {
        let input = invocation.payload.remove("input").unwrap();

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);

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
        let link: ProviderLink = invocation.payload.consume("link").unwrap();
        let result = link.to_string();

        let (send, stream) = stream(1);

        defer(vec![
          send(TransportWrapper::new("output", MessageTransport::success(&result))),
          send(TransportWrapper::done("output")),
        ]);

        Ok(stream)
      }
      "reverse" => {
        println!("Reverse payload {:?}", invocation.payload);
        let input: String = invocation.payload.consume("input").unwrap();

        let (send, stream) = stream(1);

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

        let (send, stream) = stream(1);
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

        let (send, stream) = stream(1);

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

  fn list(&self) -> &wasmflow_interface::ProviderSignature {
    &self.0
  }
}
