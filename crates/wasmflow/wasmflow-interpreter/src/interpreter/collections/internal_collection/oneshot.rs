use futures::future::BoxFuture;
use futures::StreamExt;
use serde_json::Value;
use wasmflow_packet_stream::{Packet, PacketStream, PayloadFlux};
use wasmrs_rx::Observer;

use crate::{BoxError, Operation};

#[derive(Default, Debug, Clone, Copy)]
pub struct OneShotComponent {}

impl Operation for OneShotComponent {
  fn handle(
    &self,
    payload: wasmflow_packet_stream::StreamMap,
    _data: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, BoxError>> {
    let task = async move {
      let flux = PayloadFluxChannel::new();
      let mut futs = Vec::new();
      for (port, mut stream) in payload.into_iter() {
        futs.push(async move { (port, stream.next().await) });
      }
      let fut = futures::future::join_all(futs).await;
      for (port, message) in fut {
        match message {
          Some(Ok(message)) => {
            flux.send(message);
            flux.send(Packet::done(port));
          }
          Some(Err(_e)) => {
            flux.send(Packet::component_error("Error sending oneshot payload"));
          }
          None => todo!(),
        }
      }
      Ok(PacketStream::new(flux.take_rx().unwrap()))
    };

    Box::pin(task)
  }
}
