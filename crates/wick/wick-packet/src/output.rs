use tracing::warn;
use wasmrs::{PayloadError, RawPayload};
use wasmrs_guest::{FluxChannel, Observer};

use crate::Packet;

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

  pub fn send(&mut self, value: &T) {
    self.send_raw(Packet::encode(&self.name, value));
  }

  pub fn send_raw(&mut self, value: Packet) {
    println!("Sending packet: {:?}", value);
    if let Err(e) = self.channel.send_result(value.into()) {
      warn!(
        port = self.name,
        error = %e,
        "failed sending packet on output channel, this is a bug"
      );
    };
  }

  pub fn open_bracket(&mut self) {
    self.send_raw(Packet::open_bracket(&self.name));
  }

  pub fn close_bracket(&mut self) {
    self.send_raw(Packet::close_bracket(&self.name));
  }

  pub fn done(&mut self) {
    self.send_raw(Packet::done(&self.name));
  }

  pub fn error(&mut self, err: impl AsRef<str>) {
    let _ = self.send_raw(Packet::err(&self.name, err));
  }
}
