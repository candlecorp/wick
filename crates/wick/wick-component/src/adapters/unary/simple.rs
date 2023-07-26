use tokio_stream::StreamExt;
use wasmrs_runtime::BoxFuture;
use wick_packet::Packet;

use crate::adapters::encode;
use crate::{propagate_if_error, SingleOutput, WickStream};

#[macro_export]
/// This macro will generate the implementations for simple binary operations, operations that take two inputs, produce one output, and are largely want to remain ignorant of stream state.
macro_rules! unary_simple {
  ($name:ident) => {
    #[async_trait::async_trait(?Send)]
    impl $name::Operation for Component {
      type Error = anyhow::Error;
      type Outputs = $name::Outputs;
      type Config = $name::Config;

      async fn $name(
        input: WickStream<Packet>,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        wick_component::unary::simple(input, &mut outputs, &ctx, &$name).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common binary operations that have one output.
pub async fn simple<'f, 'c, INPUT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  input: WickStream<Packet>,
  outputs: &mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'f F,
) -> Result<(), E>
where
  'f: 'static,
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(INPUT, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  INPUT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  let _ = inner::<INPUT, OUTPUT, CONTEXT, OUTPORT, F, E>(input, outputs, ctx, func).await;
  outputs.single_output().done();

  Ok(())
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn inner<'f, 'out, 'c, INPUT, OUTPUT, CONTEXT, OUTPORT, F, E>(
  mut input_stream: WickStream<Packet>,
  outputs: &'out mut OUTPORT,
  ctx: &'c CONTEXT,
  func: &'f F,
) -> WickStream<Packet>
where
  'f: 'static,
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(INPUT, CONTEXT) -> BoxFuture<Result<OUTPUT, E>> + wasmrs_runtime::ConditionallySendSync,
  OUTPORT: SingleOutput + wasmrs_runtime::ConditionallySendSync,
  INPUT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUT: serde::Serialize + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    let input = input_stream.next().await;
    if input.is_none() {
      break;
    }
    let input = input.unwrap();
    let input = propagate_if_error!(input, outputs, continue);
    if input.is_open_bracket() {
      outputs.broadcast_open();
      input_stream = inner(input_stream, outputs, ctx, func).await;
      outputs.broadcast_close();
    } else if input.is_close_bracket() || input.is_done() {
      break;
    } else {
      let input: INPUT = propagate_if_error!(input.decode(), outputs, continue);
      outputs
        .single_output()
        .send_raw_payload(encode(func(input.clone(), ctx.clone()).await));
    }
  }

  input_stream
}
