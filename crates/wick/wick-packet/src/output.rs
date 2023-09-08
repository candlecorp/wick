use tracing::warn;
use wasmrs::{PayloadError, RawPayload};
use wasmrs_runtime::ConditionallySend;
use wasmrs_rx::{FluxChannel, Observer};

use crate::{Packet, PacketPayload};

pub struct OutgoingPort<T> {
  channel: FluxChannel<RawPayload, PayloadError>,
  name: String,
  _phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Debug for OutgoingPort<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Output").field("name", &self.name).finish()
  }
}

pub trait Port: ConditionallySend {
  fn send_packet(&mut self, value: Packet);

  fn send_raw_payload(&mut self, value: PacketPayload) {
    self.send_packet(Packet::new_for_port(self.name(), value, 0));
  }

  fn name(&self) -> &str;

  fn open_bracket(&mut self) {
    self.send_packet(Packet::open_bracket(self.name()));
  }

  fn close_bracket(&mut self) {
    self.send_packet(Packet::close_bracket(self.name()));
  }

  fn done(&mut self) {
    self.send_packet(Packet::done(self.name()));
  }

  fn error(&mut self, err: &str) {
    self.send_packet(Packet::err(self.name(), err));
  }
}

pub trait ValuePort<T>: Port {
  fn send(&mut self, value: T);

  fn send_result(&mut self, value: Result<T, impl std::fmt::Display>);
}

impl<T> ValuePort<T> for OutgoingPort<T>
where
  T: serde::Serialize + ConditionallySend,
{
  fn send(&mut self, value: T) {
    self.send_packet(Packet::encode(self.name(), value));
  }

  fn send_result(&mut self, value: Result<T, impl std::fmt::Display>) {
    match value {
      Ok(value) => self.send(value),
      Err(err) => self.error(err.to_string().as_str()),
    }
  }
}

impl<T> ValuePort<&T> for OutgoingPort<T>
where
  T: serde::Serialize + ConditionallySend,
{
  fn send(&mut self, value: &T) {
    self.send_packet(Packet::encode(self.name(), value));
  }

  fn send_result(&mut self, value: Result<&T, impl std::fmt::Display>) {
    match value {
      Ok(value) => self.send(value),
      Err(err) => self.error(err.to_string().as_str()),
    }
  }
}

impl ValuePort<&str> for OutgoingPort<String> {
  fn send(&mut self, value: &str) {
    self.send_packet(Packet::encode(self.name(), value));
  }

  fn send_result(&mut self, value: Result<&str, impl std::fmt::Display>) {
    match value {
      Ok(value) => self.send(value),
      Err(err) => self.error(err.to_string().as_str()),
    }
  }
}

impl<T> Port for OutgoingPort<T>
where
  T: serde::Serialize + ConditionallySend,
{
  fn send_packet(&mut self, value: Packet) {
    let value = value.set_port(&self.name);
    if let Err(e) = self.channel.send_result(value.into()) {
      warn!(
        port = self.name,
        error = %e,
        "failed sending packet on output channel, this is a bug"
      );
    };
  }

  fn name(&self) -> &str {
    &self.name
  }
}

impl<T> OutgoingPort<T>
where
  T: serde::Serialize,
{
  pub fn new<K: Into<String>>(name: K, channel: FluxChannel<RawPayload, PayloadError>) -> Self {
    Self {
      channel,
      name: name.into(),
      _phantom: Default::default(),
    }
  }
}

/// Iterator over a mutable set of output ports
#[must_use]
pub struct OutputIterator<'a> {
  outputs: Vec<&'a mut dyn Port>,
}

impl<'a> std::fmt::Debug for OutputIterator<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OutputIterator")
      .field("outputs", &self.outputs.iter().map(|a| a.name()).collect::<Vec<_>>())
      .finish()
  }
}
impl<'a> OutputIterator<'a> {
  /// Create a new [OutputIterator]
  pub fn new(outputs: Vec<&'a mut dyn Port>) -> Self {
    Self { outputs }
  }
}

impl<'a> IntoIterator for OutputIterator<'a> {
  type Item = &'a mut dyn Port;

  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.outputs.into_iter()
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use tokio::task::JoinHandle;
  use tokio_stream::StreamExt;

  use super::*;
  use crate::{packet_stream, PacketStream};

  #[test_logger::test(tokio::test)]
  async fn test_outputs() -> Result<()> {
    struct Outputs {
      a: OutgoingPort<i32>,
      b: OutgoingPort<String>,
      c: OutgoingPort<SomeStruct>,
    }

    let (stream, rx) = FluxChannel::new_parts();

    let mut outputs = Outputs {
      a: OutgoingPort::new("a", stream.clone()),
      b: OutgoingPort::new("b", stream.clone()),
      c: OutgoingPort::new("c", stream),
    };

    #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
    struct SomeStruct {
      a: String,
    }
    let some_struct = SomeStruct { a: "hey".to_owned() };

    outputs.a.send(&42);
    outputs.a.send(42);
    outputs.b.send("hey");
    outputs.b.send("hey".to_owned());
    let kinda_string = std::borrow::Cow::Borrowed("hey");
    outputs.b.send(kinda_string.as_ref());
    outputs.c.send(&some_struct.clone());
    outputs.c.send(&some_struct);
    drop(outputs);

    let mut packets = rx.collect::<Vec<_>>().await;

    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<i32>()?, 42);
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<i32>()?, 42);
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<String>()?, "hey");
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<String>()?, "hey");
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<String>()?, "hey");
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<SomeStruct>()?, some_struct);
    let p: Packet = packets.remove(0).into();
    assert_eq!(p.decode::<SomeStruct>()?, some_struct);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_inputs() -> Result<()> {
    struct Inputs {
      #[allow(unused)]
      task: JoinHandle<()>,
      a: PacketStream,
      b: PacketStream,
    }
    impl Inputs {
      fn new(mut stream: PacketStream) -> Self {
        let (a_tx, a_rx) = PacketStream::new_channels();
        let (b_tx, b_rx) = PacketStream::new_channels();
        let task = tokio::spawn(async move {
          while let Some(next) = stream.next().await {
            let _ = match next {
              Ok(packet) => match packet.port() {
                "a" => a_tx.send(packet),
                "b" => b_tx.send(packet),
                crate::Packet::FATAL_ERROR => {
                  let _ = a_tx.send(packet.clone());
                  b_tx.send(packet.clone())
                }
                _ => continue,
              },
              Err(e) => {
                let _ = a_tx.error(e.clone());
                b_tx.error(e)
              }
            };
          }
        });

        Self { task, a: a_rx, b: b_rx }
      }
    }

    let stream = packet_stream!(("a", 32), ("b", "Hey"));

    let mut inputs = Inputs::new(stream);

    assert_eq!(inputs.a.next().await.unwrap()?.decode::<i32>()?, 32);
    assert_eq!(inputs.b.next().await.unwrap()?.decode::<String>()?, "Hey");

    Ok(())
  }
}
