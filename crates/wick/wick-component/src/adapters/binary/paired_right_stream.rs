use tokio_stream::StreamExt;
use wasmrs_runtime::BoxFuture;
use wick_packet::{BoxStream, PacketExt, VPacket};

use crate::adapters::encode;
use crate::{make_substream_window, propagate_if_error, SingleOutput, WickStream};

#[macro_export]
/// This macro will generate the implementations for binary operations that pair single packets with potential streams of packets.
///
/// A common example would be file system write() operations, where contents are paired with a filename and a stream of contents.
macro_rules! binary_paired_right_stream {
  ($name:ident => $handler:ident) => {
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
    #[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
    impl $name::Operation for Component {
      type Error = wick_component::AnyError;
      type Inputs = $name::Inputs;
      type Outputs = $name::Outputs;
      type Config = $name::Config;

      async fn $name(
        inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        use wick_packet::BinaryInputs;
        let (left, right) = inputs.both();

        wick_component::binary::paired_right_stream(left, right, &mut outputs, &ctx, &$handler).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common for binary operations that pair single packets with potential streams of packets.
pub async fn paired_right_stream<'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  left: BoxStream<VPacket<LEFT>>,
  right: BoxStream<VPacket<RIGHT>>,
  outputs: &mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> Result<(), E>
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, WickStream<RIGHT>, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  let (_, _) = inner::<LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(left, right, outputs, ctx, func).await;
  outputs.single_output().done();

  Ok(())
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn inner<'out, 'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  mut l_stream: BoxStream<VPacket<LEFT>>,
  mut r_stream: BoxStream<VPacket<RIGHT>>,
  outputs: &'out mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> (BoxStream<VPacket<LEFT>>, BoxStream<VPacket<RIGHT>>)
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, WickStream<RIGHT>, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    let Some(left) = l_stream.next().await else { break };
    if left.is_open_bracket() {
      make_substream_window!(outputs, {
        (l_stream, r_stream) = inner(l_stream, r_stream, outputs, ctx, func).await;
      });
      continue;
    }

    let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);

    let (tx, rx) = wasmrs_runtime::unbounded_channel();
    let mut started = false;
    let mut depth = 0;
    let (rv_tx, rv_rx) = wasmrs_runtime::oneshot();
    let ctx = ctx.clone();
    wasmrs_runtime::spawn("paired_right_stream", async move {
      let _ = rv_tx.send(encode(func(left, Box::pin(rx), ctx).await));
    });
    while let Some(packet) = r_stream.next().await {
      if packet.is_open_bracket() {
        if !started {
          depth += 1;
          outputs.broadcast_open();
          continue;
        }
        tracing::debug!("received open bracket while already started");
        let _ = tx.send(Err(
          wick_packet::Error::Component("Received open bracket while already started".to_owned()).into(),
        ));
        continue;
      }

      if packet.is_close_bracket() {
        depth -= 1;
        if depth == 0 {
          break;
        }
        continue;
      }

      if packet.is_done() {
        break;
      }

      if !started {
        started = true;
      }

      let _ = tx.send(packet.decode().map_err(Into::into));
    }
    drop(tx);

    match rv_rx.await {
      Ok(v) => outputs.single_output().send_raw_payload(v),
      Err(e) => outputs.broadcast_err(e.to_string()),
    }
  }

  (l_stream, r_stream)
}
