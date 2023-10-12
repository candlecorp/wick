use tokio_stream::StreamExt;
use wasmrs_runtime::BoxFuture;
use wasmrs_rx::Observer;
use wick_packet::{BoxStream, PacketExt, VPacket, WasmRsChannel};

use crate::{propagate_if_error, Broadcast};

#[macro_export]
/// This macro will generate the implementations for simple unary operations, operations that take one input and can produce any number of outputs.
macro_rules! unary_with_outputs {
  ($name:ident => $handler:ident) => {
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
    #[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
    impl $name::Operation for Component {
      type Error = wick_component::AnyError;
      type Inputs = $name::Inputs;
      type Outputs = $name::Outputs;
      type Config = $name::Config;

      async fn $name(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        let output_factory = || $name::Outputs::new_parts();
        use wick_packet::UnaryInputs;
        let input = inputs.take_input();
        wick_component::unary::with_outputs(input, outputs, output_factory, &ctx, &$handler).await?;

        Ok(())
      }
    }
  };
}

/// Operation helper for common unary operations that have one input and control their own output.
pub async fn with_outputs<'out, 'c, INPUT, OUTPUTS, FACTORY, CONTEXT, F, E>(
  input: BoxStream<VPacket<INPUT>>,
  outputs: OUTPUTS,
  output_factory: FACTORY,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> Result<(), E>
where
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  INPUT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUTS: WasmRsChannel + Broadcast + wasmrs_runtime::ConditionallySendSync,
  FACTORY: Fn() -> (
      OUTPUTS,
      wasmrs_rx::FluxReceiver<wasmrs::RawPayload, wasmrs::PayloadError>,
    ) + wasmrs_runtime::ConditionallySendSync,
  F: Fn(INPUT, OUTPUTS, CONTEXT) -> BoxFuture<Result<(), E>> + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  let _ =
    inner::<INPUT, OUTPUTS, FACTORY, CONTEXT, F, E>(input, outputs, std::sync::Arc::new(output_factory), ctx, func)
      .await;

  Ok(())
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn inner<'out, 'c, INPUT, OUTPUTS, FACTORY, CONTEXT, F, E>(
  mut input_stream: BoxStream<VPacket<INPUT>>,
  mut outputs: OUTPUTS,
  output_factory: std::sync::Arc<FACTORY>,
  ctx: &'c CONTEXT,
  func: &'static F,
) -> (BoxStream<VPacket<INPUT>>, OUTPUTS)
where
  INPUT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
  OUTPUTS: WasmRsChannel + Broadcast + wasmrs_runtime::ConditionallySendSync,
  FACTORY: Fn() -> (
      OUTPUTS,
      wasmrs_rx::FluxReceiver<wasmrs::RawPayload, wasmrs::PayloadError>,
    ) + wasmrs_runtime::ConditionallySendSync,
  CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
  F: Fn(INPUT, OUTPUTS, CONTEXT) -> BoxFuture<Result<(), E>> + wasmrs_runtime::ConditionallySendSync,
  E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
{
  loop {
    let Some(input) = input_stream.next().await else { break };
    if input.is_open_bracket() {
      outputs.broadcast_open();
      let (inner_outputs, mut inner_output_rx) = output_factory();
      (input_stream, outputs) = inner(input_stream, inner_outputs, output_factory.clone(), ctx, func).await;
      while let Some(payload) = inner_output_rx.next().await {
        let _ = outputs.channel().send_result(payload);
      }
      outputs.broadcast_close();
    } else if input.is_close_bracket() || input.is_done() {
      break;
    } else {
      let input: INPUT = propagate_if_error!(input.decode(), outputs, continue);
      let (inner_outputs, mut inner_output_rx) = output_factory();
      let result = func(input.clone(), inner_outputs, ctx.clone()).await;
      while let Some(payload) = inner_output_rx.next().await {
        let _ = outputs.channel().send_result(payload);
      }
      propagate_if_error!(result, outputs, continue);
    }
  }

  (input_stream, outputs)
}
