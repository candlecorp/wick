use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + 'a>>;
use flow_component::{Component, ComponentError, RuntimeCallback};
use futures::{Future, StreamExt};
use seeded_random::{Random, Seed};
use tokio::spawn;
use tokio::task::JoinHandle;
use tracing::trace;
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::{ComponentSignature, OperationSignature, TypeSignature};
use wick_packet::{fan_out, packet_stream, ComponentReference, Invocation, Packet, PacketStream};

pub struct TestComponent(ComponentSignature);
impl TestComponent {
  #[allow(dead_code)]
  pub fn new() -> Self {
    let signature = ComponentSignature::new("test-component")
      .version("0.0.0")
      .metadata(Default::default())
      .add_operation(
        OperationSignature::new("echo")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("call")
          .add_input("component", TypeSignature::Link { schemas: vec![] })
          .add_input("message", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("uppercase")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("reverse")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("add")
          .add_input("left", TypeSignature::U64)
          .add_input("right", TypeSignature::U64)
          .add_output("output", TypeSignature::U64),
      )
      .add_operation(
        OperationSignature::new("timeout")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("timeout-nodone")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("concat")
          .add_input("left", TypeSignature::String)
          .add_input("right", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("noimpl")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("component_error")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("concat-five")
          .add_input("one", TypeSignature::String)
          .add_input("two", TypeSignature::String)
          .add_input("three", TypeSignature::String)
          .add_input("four", TypeSignature::String)
          .add_input("five", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("splitter")
          .add_input("input", TypeSignature::String)
          .add_output(
            "rest",
            TypeSignature::Stream {
              ty: Box::new(TypeSignature::String),
            },
          )
          .add_output(
            "vowels",
            TypeSignature::Stream {
              ty: Box::new(TypeSignature::String),
            },
          ),
      )
      .add_operation(
        OperationSignature::new("join")
          .add_input(
            "input",
            TypeSignature::Stream {
              ty: Box::new(TypeSignature::String),
            },
          )
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("ref_to_string")
          .add_input("link", TypeSignature::Link { schemas: vec![] })
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("exception")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("panic")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("error")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      )
      .add_operation(
        OperationSignature::new("copy")
          .add_input("input", TypeSignature::String)
          .add_input("times", TypeSignature::U64)
          .add_output(
            "output",
            TypeSignature::Stream {
              ty: Box::new(TypeSignature::String),
            },
          ),
      )
      .add_operation(OperationSignature::new("no-inputs").add_output("output", TypeSignature::String))
      .add_operation(
        OperationSignature::new("render")
          .add_input("input", TypeSignature::String)
          .add_output("output", TypeSignature::String),
      );

    Self(signature)
  }
}

type Sender = Box<dyn FnMut(Packet) -> JoinHandle<()> + Send + Sync>;

fn stream(_seed: u64) -> (Sender, PacketStream) {
  let flux = FluxChannel::new();

  let stream = PacketStream::new(Box::new(flux.take_rx().unwrap()));
  let mut total = 0;
  let sender: Sender = Box::new(move |msg: Packet| {
    let rng = Random::new();
    let delay = rng.range(10, 200);
    total += delay;
    let tx = flux.clone();
    tokio::spawn(async move {
      trace!(total, "sleeping for delayed send");
      tokio::time::sleep(Duration::from_millis(total.into())).await;
      let _ = tx.send(msg);
    })
  });
  (sender, stream)
}

fn defer(futs: Vec<impl Future<Output = Result<(), impl std::error::Error + Send + Sync>> + Sync + Send + 'static>) {
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

impl Component for TestComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _config: Option<wick_packet::OperationConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let operation = invocation.target.operation_id();
    println!("got op {} in echo test collection", operation);
    Box::pin(async move { Ok(handler(invocation, stream, callback)?) })
  }

  fn list(&self) -> &ComponentSignature {
    &self.0
  }
}

fn handler(
  invocation: Invocation,
  mut payload_stream: PacketStream,
  callback: Arc<RuntimeCallback>,
) -> anyhow::Result<PacketStream> {
  let operation = invocation.target.operation_id();
  match operation {
    "echo" => {
      let (mut send, stream) = stream(1);
      println!("got echo");
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        println!("got echo: waiting for payload");
        while let Some(Ok(payload)) = input.next().await {
          println!("got echo: got payload");
          defer(vec![send(payload.set_port("output"))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "call" => {
      let (mut send, stream) = stream(1);
      println!("got echo");
      spawn(async move {
        let (mut message, mut component) = fan_out!(payload_stream, "message", "component");
        println!("got echo: waiting for payload");
        while let (Some(Ok(message)), Some(Ok(component))) = (message.next().await, component.next().await) {
          println!("got echo: got compref: {:?}", component.payload());
          let link: ComponentReference = component.deserialize().unwrap();
          let message: String = message.deserialize().unwrap();
          let packets = packet_stream!(("input", message));
          let mut response = callback(link, "reverse".to_owned(), packets, None, None).await.unwrap();
          while let Some(Ok(res)) = response.next().await {
            defer(vec![send(res.set_port("output"))]);
          }
        }
      });
      Ok(stream)
    }
    "reverse" => {
      let (mut send, stream) = stream(1);
      println!("got reverse");
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        println!("got reverse: waiting for payload");
        while let Some(Ok(payload)) = input.next().await {
          println!("got reverse: got payload");
          let msg: String = payload.deserialize().unwrap();
          defer(vec![send(Packet::encode(
            "output",
            &msg.chars().rev().collect::<String>(),
          ))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "uppercase" => {
      let (mut send, stream) = stream(1);
      println!("got uppercase");
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        println!("got uppercase: waiting for payload");
        while let Some(Ok(payload)) = input.next().await {
          println!("got uppercase: got payload");
          let msg: String = payload.deserialize().unwrap();
          defer(vec![send(Packet::encode("output", &msg.to_ascii_uppercase()))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "add" => {
      let (mut send, stream) = stream(1);
      spawn(async move {
        let (mut left, mut right) = fan_out!(payload_stream, "left", "right");
        while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
          let left: u64 = left.deserialize().unwrap();
          let right: u64 = right.deserialize().unwrap();
          defer(vec![send(Packet::encode("output", left + right))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "copy" => {
      let (mut input, mut times) = fan_out!(payload_stream, "input", "times");
      let (mut send, stream) = stream(1);
      let task = async move {
        while let (Some(input), Some(times)) = (input.next().await, times.next().await) {
          let input: String = input?.deserialize()?;
          let times: u64 = times?.deserialize()?;
          let mut messages = Vec::new();
          for _ in 0..times {
            messages.push(send(Packet::encode("output", &input)));
          }
          messages.push(send(Packet::done("output")));
          defer(messages);
        }
        Ok::<_, wick_packet::Error>(())
      };
      defer(vec![task]);
      Ok(stream)
    }
    "splitter" => {
      let mut input = fan_out!(payload_stream, "input");
      let (mut send, stream) = stream(1);
      let task = async move {
        while let Some(input) = input.next().await {
          let input: String = input?.deserialize()?;
          let vowels: Vec<_> = input
            .chars()
            .filter(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            .collect();
          let rest: Vec<_> = input
            .chars()
            .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            .collect();

          let mut vowel_packets = vowels.iter().map(|v| Packet::encode("vowels", v)).collect::<Vec<_>>();
          vowel_packets.push(Packet::done("vowels"));

          let mut rest_packets = rest.iter().map(|v| Packet::encode("rest", v)).collect::<Vec<_>>();
          rest_packets.push(Packet::done("rest"));
          println!("{:#?}", vowel_packets);
          println!("{:#?}", rest_packets);
          defer(vowel_packets.into_iter().map(&mut send).collect());
          defer(rest_packets.into_iter().map(&mut send).collect());
        }
        Ok::<_, wick_packet::Error>(())
      };
      defer(vec![task]);
      Ok(stream)
    }
    "panic" => {
      // let input = invocation.payload.remove("input").unwrap();
      // println!("test::panic got {}", input);
      panic!();
    }
    "error" => Err(anyhow!("This operation always errors")),
    "noimpl" => {
      let (_send, stream) = stream(1);
      Ok(stream)
    }
    "component_error" => {
      let (mut send, stream) = stream(1);
      send(Packet::component_error("Oh no"));
      Ok(stream)
    }
    "timeout-nodone" => {
      let mut input = fan_out!(payload_stream, "input");
      let (mut send, stream) = stream(1);
      let task = async move {
        while let Some(input) = input.next().await {
          let input = input?;
          defer(vec![send(input.set_port("output"))]);
        }
        Ok::<_, wick_packet::Error>(())
      };
      defer(vec![task]);

      Ok(stream)
    }

    "timeout" => {
      let (_send, stream) = stream(1);
      Ok(stream)
    }

    // "concat" => {
    //   let left: String = invocation.payload.take("left").unwrap();
    //   let right: String = invocation.payload.take("right").unwrap();
    //   let result = format!("{}{}", left, right);

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new("output", MessageTransport::success(&result))),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "concat-five" => {
    //   let one: String = invocation.payload.take("one").unwrap();
    //   let two: String = invocation.payload.take("two").unwrap();
    //   let three: String = invocation.payload.take("three").unwrap();
    //   let four: String = invocation.payload.take("four").unwrap();
    //   let five: String = invocation.payload.take("five").unwrap();
    //   let result = format!("{}{}{}{}{}", one, two, three, four, five);

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new("output", MessageTransport::success(&result))),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "timeout-nodone" => {
    //   let input: String = invocation.payload.take("input").unwrap();

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new("output", MessageTransport::success(&input))),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "join" => {
    //   let left: String = invocation.payload.take("left").unwrap();
    //   let right: String = invocation.payload.take("right").unwrap();
    //   let result = format!("{}{}", left, right);

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new("output", MessageTransport::success(&result))),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "ref_to_string" => {
    //   let link: ComponentReference = invocation.payload.take("link").unwrap();
    //   let result = link.to_string();

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new("output", MessageTransport::success(&result))),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "no-inputs" => {
    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new(
    //       "output",
    //       MessageTransport::success(&"Hello world".to_owned()),
    //     )),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "reverse" => {
    //   println!("Reverse payload {:?}", invocation.payload);
    //   let input: String = invocation.payload.take("input").unwrap();

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new(
    //       "output",
    //       MessageTransport::success(&input.chars().rev().collect::<String>()),
    //     )),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "copy" => {
    //   println!("Reverse payload {:?}", invocation.payload);
    //   let input: String = invocation.payload.take("input").unwrap();
    //   let times: u64 = invocation.payload.take("times").unwrap();
    //   let mut futs = vec![];

    //   let (mut send, stream) = stream(1);
    //   for _ in 0..times {
    //     futs.push(send(TransportWrapper::new("output", MessageTransport::success(&input))));
    //   }

    //   futs.push(send(TransportWrapper::done("output")));

    //   defer(futs);

    //   Ok(stream)
    // }
    // "exception" => {
    //   let input = invocation.payload.remove("input").unwrap();
    //   println!("test::exception got {}", input);

    //   let (mut send, stream) = stream(1);

    //   defer(vec![
    //     send(TransportWrapper::new(
    //       "output",
    //       MessageTransport::Failure(Failure::Exception("test::exception".to_owned())),
    //     )),
    //     send(TransportWrapper::done("output")),
    //   ]);

    //   Ok(stream)
    // }
    // "panic" => {
    //   let input = invocation.payload.remove("input").unwrap();
    //   println!("test::panic got {}", input);
    //   panic!();
    // }
    _ => Err(anyhow::anyhow!("Operation {} not handled", operation)),
  }
}
