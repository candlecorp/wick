use tokio_stream::StreamExt;
use wasmrs_runtime::BoxFuture;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::Packet;

use crate::adapters::encode;
use crate::{await_next_ok_or, make_substream_window, propagate_if_error, SingleOutput, WickStream};

#[macro_export]
/// This macro will generate the implementations for binary operations that pair single packets with potential streams of packets.
///
/// A common example would be file system write() operations, where contents are paired with a filename and a stream of contents.
macro_rules! binary_paired_right_stream {
  ($name:ident) => {
    #[async_trait::async_trait(?Send)]
    impl $name::Operation for Component {
      type Error = anyhow::Error;
      type Outputs = $name::Outputs;
      type Config = $name::Config;

      async fn $name(
        left: WickStream<Packet>,
        right: WickStream<Packet>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        wick_component::binary::paired_right_stream(left, right, &mut outputs, &ctx, &$name).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common for binary operations that pair single packets with potential streams of packets.
pub async fn paired_right_stream<'f, 'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  left: WickStream<Packet>,
  right: WickStream<Packet>,
  outputs: &mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'f F,
) -> Result<(), E>
where
  'f: 'static,
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
async fn inner<'f, 'out, 'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  mut l_stream: WickStream<Packet>,
  mut r_stream: WickStream<Packet>,
  outputs: &'out mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'f F,
) -> (WickStream<Packet>, WickStream<Packet>)
where
  'f: 'static,
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, WickStream<RIGHT>, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    let left = await_next_ok_or!(l_stream, outputs, continue);
    if left.is_open_bracket() {
      make_substream_window!(outputs, {
        (l_stream, r_stream) = inner(l_stream, r_stream, outputs, ctx, func).await;
      });
      continue;
    }

    let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);

    let (tx, rx) = FluxChannel::new_parts();
    let mut started = false;
    let mut depth = 0;
    let (rv_tx, rv_rx) = wasmrs_runtime::oneshot();
    let ctx = ctx.clone();
    wasmrs_runtime::spawn("paired_right_stream", async move {
      let _ = rv_tx.send(encode(func(left, rx.boxed(), ctx).await));
    });
    while let Some(packet) = r_stream.next().await {
      if let Err(e) = packet {
        let _ = tx.send_result(Err(e));
        continue;
      }
      let packet = packet.unwrap();

      if packet.is_open_bracket() {
        if !started {
          depth += 1;
          outputs.broadcast_open();
          continue;
        }
        tracing::debug!("Received open bracket while already started");
        let _ = tx.send_result(Err(Box::new(wick_packet::Error::Component(
          "Received open bracket while already started".to_owned(),
        ))));
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

      let _ = tx.send_result(packet.decode::<RIGHT>().map_err(Into::into));
    }
    drop(tx);

    match rv_rx.await {
      Ok(v) => outputs.single_output().send_raw_payload(v),
      Err(e) => outputs.broadcast_err(e.to_string()),
    }
  }

  (l_stream, r_stream)
}
