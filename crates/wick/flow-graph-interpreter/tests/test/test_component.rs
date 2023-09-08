use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + 'a>>;
use flow_component::{Component, ComponentError, RuntimeCallback};
use futures::{Future, StreamExt};
use seeded_random::{Random, Seed};
use tokio::spawn;
use tokio::task::JoinHandle;
use tracing::{trace, Span};
use wasmrs_rx::{FluxChannel, Observer};
use wick_interface_types::{ComponentSignature, OperationSignature, Type};
use wick_packet::{
  fan_out,
  packet_stream,
  ComponentReference,
  InherentData,
  Invocation,
  Packet,
  PacketStream,
  RuntimeConfig,
};

pub struct TestComponent(ComponentSignature);
impl TestComponent {
  #[allow(dead_code)]
  pub fn new() -> Self {
    let signature = ComponentSignature::new_named("test-component")
      .set_version("0.0.0")
      .metadata(Default::default())
      .add_operation(
        OperationSignature::new_named("echo")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(OperationSignature::new_named("empty_stream").add_output("output", Type::String))
      .add_operation(
        OperationSignature::new_named("wait")
          .add_input("input", Type::U64)
          .add_output("output", Type::U64),
      )
      .add_operation(
        #[allow(deprecated)]
        OperationSignature::new_named("call")
          .add_input("component", Type::Link { schemas: vec![] })
          .add_input("message", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("uppercase")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("reverse")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("add")
          .add_input("left", Type::U64)
          .add_input("right", Type::U64)
          .add_output("output", Type::U64),
      )
      .add_operation(
        OperationSignature::new_named("timeout")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("timeout-nodone")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("concat")
          .add_input("left", Type::String)
          .add_input("right", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("noimpl")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("component_error")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("concat-five")
          .add_input("one", Type::String)
          .add_input("two", Type::String)
          .add_input("three", Type::String)
          .add_input("four", Type::String)
          .add_input("five", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("splitter")
          .add_input("input", Type::String)
          .add_output("rest", Type::String)
          .add_output("vowels", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("join")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        #[allow(deprecated)]
        OperationSignature::new_named("ref_to_string")
          .add_input("link", Type::Link { schemas: vec![] })
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("exception")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("panic")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("error")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
      )
      .add_operation(
        OperationSignature::new_named("copy")
          .add_input("input", Type::String)
          .add_input("times", Type::U64)
          .add_output("output", Type::String),
      )
      .add_operation(OperationSignature::new_named("no-inputs").add_output("output", Type::String))
      .add_operation(
        OperationSignature::new_named("render")
          .add_input("input", Type::String)
          .add_output("output", Type::String),
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
      trace!(packet=?msg, "sending packet in test");
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
    _config: Option<RuntimeConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let operation = invocation.target.operation_id();
    println!("got op {} in test collection", operation);
    Box::pin(async move { Ok(handler(invocation, callback)?) })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.0
  }
}

fn handler(invocation: Invocation, callback: Arc<RuntimeCallback>) -> anyhow::Result<PacketStream> {
  let mut payload_stream = invocation.packets;
  let operation = invocation.target.operation_id().to_owned();
  println!("handling {}", operation);
  let (mut send, stream) = stream(1);

  macro_rules! break_if_done {
    ($($id:ident),*) => {
      $(
        if $id.is_done() { break; }
      )*
    };
  }

  macro_rules! continue_if_bracket {
    ($($id:ident),*, $($output:literal),*) => {
      let ports = vec![$($output),*];
      $(
        if $id.is_bracket() {
        for port in ports {
            defer(vec![send($id.clone().set_port(port))]);
        }
        continue;
      }
      )*
    }
  }

  match operation.as_str() {
    "echo" => {
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        while let Some(Ok(payload)) = input.next().await {
          break_if_done!(payload);
          defer(vec![send(payload.set_port("output"))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "empty_stream" => {
      spawn(async move {
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "wait" => {
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        println!("got echo: waiting for payload");
        while let Some(Ok(payload)) = input.next().await {
          break_if_done!(payload);
          continue_if_bracket!(payload, "output");

          let millis = payload.decode::<u64>().unwrap();
          let before = SystemTime::now();
          tokio::time::sleep(Duration::from_millis(millis)).await;
          let slept_for = SystemTime::now().duration_since(before).unwrap().as_millis() as u64;
          println!("slept for {}ms", slept_for);
          defer(vec![send(Packet::encode("output", slept_for))]);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "call" => {
      spawn(async move {
        let (mut message, mut component) = fan_out!(payload_stream, "message", "component");
        while let (Some(Ok(message)), Some(Ok(component))) = (message.next().await, component.next().await) {
          break_if_done!(message);
          break_if_done!(component);
          continue_if_bracket!(message, "output");
          continue_if_bracket!(component, "output");

          println!("got compref: {:?}", component.payload());
          let link: ComponentReference = component.decode().unwrap();
          let message: String = message.decode().unwrap();
          let packets = packet_stream!(("input", message));
          let mut response = callback(
            link,
            "reverse".to_owned(),
            packets,
            InherentData::unsafe_default(),
            Default::default(),
            &Span::current(),
          )
          .await
          .unwrap();
          while let Some(Ok(res)) = response.next().await {
            defer(vec![send(res.set_port("output"))]);
          }
        }
      });
      Ok(stream)
    }
    "reverse" => {
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        while let Some(packet) = input.next().await {
          let packet = packet.unwrap();
          println!("got packet in {} op: {:?}", operation, packet);
          if packet.port() != "input" {
            panic!("invalid port: {:?}", packet);
          }
          break_if_done!(packet);
          continue_if_bracket!(packet, "output");

          let msg: String = packet.decode().unwrap();
          let _ = send(Packet::encode("output", &msg.chars().rev().collect::<String>())).await;
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "uppercase" => {
      spawn(async move {
        let mut input = fan_out!(payload_stream, "input");
        while let Some(packet) = input.next().await {
          let packet = packet.unwrap();
          println!("got packet in {} op: {:?}", operation, packet);
          if packet.port() != "input" {
            panic!("invalid port: {:?}", packet);
          }
          break_if_done!(packet);
          continue_if_bracket!(packet, "output");

          let msg: String = packet.decode().unwrap();
          let _ = send(Packet::encode("output", &msg.to_ascii_uppercase())).await;
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "concat" => {
      spawn(async move {
        let (mut left, mut right) = fan_out!(payload_stream, "left", "right");
        while let (Some(left), Some(right)) = (left.next().await, right.next().await) {
          match (left, right) {
            (Ok(left), Ok(right)) => {
              let mut packets = Vec::new();
              break_if_done!(left, right);
              continue_if_bracket!(left, right, "output");

              let left: String = left.decode().unwrap();
              let right: String = right.decode().unwrap();
              packets.push(send(Packet::encode("output", format!("{}{}", left, right))));

              defer(packets);
            }
            (Err(e), _) => {
              defer(vec![send(Packet::err("output", e.to_string()))]);
            }
            (_, Err(e)) => {
              defer(vec![send(Packet::err("output", e.to_string()))]);
            }
          }
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "add" => {
      spawn(async move {
        let (mut left, mut right) = fan_out!(payload_stream, "left", "right");
        while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
          break_if_done!(left, right);
          continue_if_bracket!(left, right, "output");

          let mut packets = Vec::new();
          let left: u64 = left.decode().unwrap();
          let right: u64 = right.decode().unwrap();
          packets.push(send(Packet::encode("output", left + right)));

          defer(packets);
        }
        defer(vec![send(Packet::done("output"))]);
      });
      Ok(stream)
    }
    "copy" => {
      let (mut input, mut times) = fan_out!(payload_stream, "input", "times");

      let task = async move {
        while let (Some(Ok(input)), Some(Ok(times))) = (input.next().await, times.next().await) {
          break_if_done!(input, times);
          continue_if_bracket!(input, times, "output");

          let mut packets = Vec::new();
          let input: String = input.decode()?;
          let times: u64 = times.decode()?;
          for _ in 0..times {
            packets.push(send(Packet::encode("output", &input)));
          }
          packets.push(send(Packet::done("output")));
          defer(packets);
        }
        Ok::<_, wick_packet::Error>(())
      };
      defer(vec![task]);
      Ok(stream)
    }
    "splitter" => {
      let mut input = fan_out!(payload_stream, "input");

      let task = async move {
        while let Some(Ok(input)) = input.next().await {
          break_if_done!(input);
          continue_if_bracket!(input, "rest", "vowels");

          let input: String = input.decode()?;
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
    "noimpl" => Ok(stream),
    "component_error" => {
      send(Packet::component_error("Oh no"));
      Ok(stream)
    }
    "timeout-nodone" => {
      let mut input = fan_out!(payload_stream, "input");

      let task = async move {
        while let Some(input) = input.next().await {
          let input = input?;
          break_if_done!(input);
          defer(vec![send(input.set_port("output"))]);
        }
        Ok::<_, wick_packet::Error>(())
      };
      defer(vec![task]);

      Ok(stream)
    }

    "timeout" => Ok(stream),

    _ => Err(anyhow::anyhow!("Operation {} not handled", operation)),
  }
}
