use tokio_stream::StreamExt;
use wick_packet::Packet;

use crate::adapters::encode;
use crate::{
  await_next_ok_or,
  if_done_close_then,
  make_substream_window,
  propagate_if_error,
  SingleOutput,
  WickStream,
};

#[macro_export]
/// This macro will generate the implementations for simple binary operations, operations that take two inputs, produce one output, and are largely want to remain ignorant of stream state.
macro_rules! binary_interleaved_pairs {
  ($name:ident) => {
    #[async_trait::async_trait(?Send)]
    impl $name::Operation for Component {
      type Error = Box<dyn std::error::Error + 'static>;
      type Outputs = $name::Outputs;
      type Config = $name::Config;

      async fn $name(
        left: WickStream<Packet>,
        right: WickStream<Packet>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        wick_component::binary::interleaved_pairs(left, right, &mut outputs, &ctx, $name).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common binary operations that have one output.
pub async fn interleaved_pairs<'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  left: WickStream<Packet>,
  right: WickStream<Packet>,
  outputs: &mut OUTPORT,
  ctx: &'c CONTEXT,
  func: F,
) -> Result<(), E>
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, RIGHT, CONTEXT) -> Result<OUTPUT, E> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  let (_, _) = inner::<LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(None, None, left, right, outputs, ctx, &func).await;
  outputs.single_output().done();

  Ok(())
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn inner<'f, 'out, 'c, LEFT, RIGHT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  last_left: Option<LEFT>,
  last_right: Option<RIGHT>,
  mut l_stream: WickStream<Packet>,
  mut r_stream: WickStream<Packet>,
  outputs: &'out mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'f F,
) -> (WickStream<Packet>, WickStream<Packet>)
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(LEFT, RIGHT, CONTEXT) -> Result<OUTPUT, E> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  LEFT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  RIGHT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    match (&last_left, &last_right) {
      (Some(left), None) => {
        let right = await_next_ok_or!(r_stream, outputs, continue);
        if_done_close_then!([right], break);

        if right.is_open_bracket() {
          make_substream_window!(outputs, {
            (l_stream, r_stream) = inner(Some(left.clone()), None, l_stream, r_stream, outputs, ctx, func).await;
          });
        } else {
          let right: RIGHT = propagate_if_error!(right.decode(), outputs, continue);
          outputs
            .single_output()
            .send_raw_payload(encode(func(left.clone(), right, ctx.clone())));
        }
      }
      (None, Some(right)) => {
        let left = await_next_ok_or!(l_stream, outputs, continue);
        if_done_close_then!([left], break);

        if left.is_open_bracket() {
          make_substream_window!(outputs, {
            (l_stream, r_stream) = inner(None, Some(right.clone()), l_stream, r_stream, outputs, ctx, func).await;
          });
        } else {
          let left: LEFT = propagate_if_error!(left.decode(), outputs, continue);
          outputs
            .single_output()
            .send_raw_payload(encode(func(left, right.clone(), ctx.clone())));
        }
      }
      (None, None) => {
        let left = await_next_ok_or!(l_stream, outputs, continue);
        let right = await_next_ok_or!(r_stream, outputs, continue);

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
              .send_raw_payload(encode(func(left, right, ctx.clone())));
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
