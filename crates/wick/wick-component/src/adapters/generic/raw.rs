#[macro_export]
/// This macro will generate the implementations for operations, passing through packets without processing.
macro_rules! generic_raw {
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
        outputs: Self::Outputs,
        ctx: Context<Self::Config>,
      ) -> Result<(), Self::Error> {
        $handler(inputs, outputs, ctx).await
      }
    }
  };
}

// /// Operation helper for common unary operations that have one input and control their own output.
// pub async fn raw<'out, 'c, INPUT, OUTPUTS, CONTEXT, F, E>(
//   input: WickStream<Packet>,
//   outputs: OUTPUTS,
//   ctx: &'c CONTEXT,
//   func: &'static F,
// ) -> Result<(), E>
// where
//   CONTEXT: Clone + wasmrs_runtime::ConditionallySendSync,
//   INPUT: serde::de::DeserializeOwned + Clone + wasmrs_runtime::ConditionallySendSync,
//   OUTPUTS: WasmRsChannel + Broadcast + wasmrs_runtime::ConditionallySendSync,
//   F: Fn(INPUT, OUTPUTS, CONTEXT) -> BoxFuture<Result<(), E>> + wasmrs_runtime::ConditionallySendSync,
//   E: std::fmt::Display + wasmrs_runtime::ConditionallySendSync,
// {
//   let _ = inner::<INPUT, OUTPUTS, CONTEXT, F, E>(input, outputs, ctx, func).await;

//   Ok(())
// }
