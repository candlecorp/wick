use tokio_stream::StreamExt;
use wick_packet::{BoxStream, PacketExt, VPacket};

use crate::adapters::encode;
use crate::runtime::BoxFuture;
use crate::{if_done_close_then, make_substream_window, propagate_if_error, runtime as wasmrs_runtime, SingleOutput};

#[macro_export]
/// This macro will generate the implementations for simple binary operations, operations that take two inputs, produce one output, and are largely want to remain ignorant of stream state.
macro_rules! binary_interleaved_pairs {
  ($name:ident => $handler:ident) => {
    #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
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
        wick_component::binary::interleaved_pairs(left, right, &mut outputs, &ctx, &$handler).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common binary operations that have one output.
pub async fn interleaved_pairs<'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  left: BoxStream<VPacket<LEFT>>,
  right: BoxStream<VPacket<RIGHT>>,
  outputs: &mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> Result<(), E>
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, RIGHT, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  let (_, _) = inner::<LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(None, None, left, right, outputs, ctx, func).await;
  outputs.single_output().done();

  Ok(())
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn inner<'out, 'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  last_left: Option<LEFT>,
  last_right: Option<RIGHT>,
  mut l_stream: BoxStream<VPacket<LEFT>>,
  mut r_stream: BoxStream<VPacket<RIGHT>>,
  outputs: &'out mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> (BoxStream<VPacket<LEFT>>, BoxStream<VPacket<RIGHT>>)
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, RIGHT, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    match (&last_left, &last_right) {
      (Some(left), None) => {
        let Some(right) = r_stream.next().await else { break };

        if_done_close_then!([right], break);

        if right.is_open_bracket() {
          make_substream_window!(outputs, {
            (l_stream, r_stream) = inner(Some(left.clone()), None, l_stream, r_stream, outputs, ctx, func).await;
          });
        } else {
          let right: RIGHT = propagate_if_error!(right.decode(), outputs, continue);
          outputs
            .single_output()
            .send_raw_payload(encode(func(left.clone(), right, ctx.clone()).await));
        }
      }
      (None, Some(right)) => {
        let Some(left) = l_stream.next().await else { break };

        if_done_close_then!([left], break);

        if left.is_open_bracket() {
          make_substream_window!(outputs, {
            (l_stream, r_stream) = inner(None, Some(right.clone()), l_stream, r_stream, outputs, ctx, func).await;
          });
        } else {
          let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);
          outputs
            .single_output()
            .send_raw_payload(encode(func(left, right.clone(), ctx.clone()).await));
        }
      }
      (None, None) => {
        let Some(left) = l_stream.next().await else { break };
        let Some(right) = r_stream.next().await else { break };

        match (left.is_open_bracket(), right.is_open_bracket()) {
          (true, true) => {
            make_substream_window!(outputs, {
              (l_stream, r_stream) = inner(None, None, l_stream, r_stream, outputs, ctx, func).await;
            });
          }
          (true, false) => {
            if_done_close_then!([right], break);

            let right: RIGHT = propagate_if_error!(right.decode(), outputs, continue);
            make_substream_window!(outputs, {
              (l_stream, r_stream) = inner(None, Some(right), l_stream, r_stream, outputs, ctx, func).await;
            });
          }
          (false, true) => {
            if_done_close_then!([left], break);

            let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);
            make_substream_window!(outputs, {
              (l_stream, r_stream) = inner(Some(left), None, l_stream, r_stream, outputs, ctx, func).await;
            });
          }
          (false, false) => {
            if_done_close_then!([left, right], break);
            let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);
            let right: RIGHT = propagate_if_error!(right.decode(), outputs, continue);
            outputs
              .single_output()
              .send_raw_payload(encode(func(left, right, ctx.clone()).await));
          }
        }
      }
      (Some(_), Some(_)) => {
        unreachable!()
      }
    }
  }

  (l_stream, r_stream)
}
