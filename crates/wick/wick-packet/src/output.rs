use tracing::warn;
use wasmrs::{PayloadError, RawPayload};
use wasmrs_runtime::ConditionallySend;
use wasmrs_rx::{FluxChannel, Observer};

use crate::{Packet, PacketPayload};

pub struct Output<T>
where
  T: serde::Serialize,
{
  channel: FluxChannel<RawPayload, PayloadError>,
  name: String,
  _phantom: std::marker::PhantomData<T>,
}

impl std::fmt::Debug for Output<()> {
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

pub trait ValuePort<T>: Port
where
  T: serde::Serialize,
{
  fn send(&mut self, value: &T) {
    self.send_packet(Packet::encode(self.name(), value));
  }

  fn send_result(&mut self, value: Result<T, impl std::fmt::Display>) {
    match value {
      Ok(value) => self.send(&value),
      Err(err) => self.error(err.to_string().as_str()),
    }
  }
}

impl<T> Port for Output<T>
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

impl<T> ValuePort<T> for Output<T> where T: serde::Serialize + ConditionallySend {}

impl<T> Output<T>
where
  T: serde::Serialize,
{
  pub fn new(name: impl AsRef<str>, channel: FluxChannel<RawPayload, PayloadError>) -> Self {
    Self {
      channel,
      name: name.as_ref().to_owned(),
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
