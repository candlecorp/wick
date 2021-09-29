use std::path::PathBuf;

use structopt::StructOpt;
use tap::{
  TestBlock,
  TestRunner,
};
use tokio_stream::StreamExt;
use vino_provider::native::prelude::{
  Entity,
  TransportMap,
};
use vino_provider_cli::LoggingOptions;
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::{
  Failure,
  MessageTransport,
  Success,
};

use super::WasiOptions;
use crate::error::VowError;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[structopt(flatten)]
  logging: LoggingOptions,

  #[structopt(flatten)]
  pull: super::PullOptions,

  #[structopt(flatten)]
  wasi: WasiOptions,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  /// The path to the data file.
  data_path: PathBuf,
}
#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  vino_provider_cli::init_logging(&opts.logging)?;

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::try_load(&component, 1, Some((&opts.wasi).into()), None)?;

  let data = vino_test::read_data(opts.data_path.clone())
    .map_err(|e| VowError::NotFound(opts.data_path.clone(), e.to_string()))?;

  let mut harness = TestRunner::new(Some(format!(
    "Vow test for : {}",
    opts.data_path.to_string_lossy()
  )));

  for test in data {
    let mut payload = TransportMap::new();
    let entity = Entity::component_direct(test.component.clone());
    for (k, v) in &test.inputs {
      payload.insert(k, MessageTransport::Success(Success::Serialized(v.clone())));
    }

    let stream = provider
      .invoke(entity, payload)
      .await
      .map_err(VowError::ComponentPanic)?;
    let outputs: Vec<_> = stream.collect().await;
    let description = test
      .description
      .map_or_else(String::new, |desc| format!(" - {}", desc));
    let mut test_block = TestBlock::new(Some(format!(
      "Component '{}'{}",
      test.component, description
    )));

    for (i, output) in test.outputs.into_iter().enumerate() {
      let result = outputs[i].port == output.port;
      test_block.add_test(
        move || result,
        format!("Output port name is '{}'", output.port),
        make_diagnostic(&outputs[i].port, &output.port),
      );

      if let Some(value) = &output.payload.value {
        let actual_payload = outputs[i].payload.clone();

        let actual_value: Result<serde_value::Value> =
          actual_payload.try_into().map_err(VowError::TransportError);
        let expected_value = value.clone();

        let diagnostic = Some(vec![
          format!(
            "Actual: {:?}",
            match &actual_value {
              Ok(v) => format!("{:?}", v),
              Err(e) => format!("Could not deserialize payload, message was : {}", e),
            }
          ),
          format!("Expected: {:?}", expected_value),
        ]);

        test_block.add_test(
          move || match actual_value {
            Ok(val) => val == expected_value,
            Err(_e) => false,
          },
          "Payload value matches",
          diagnostic,
        );
      }
      if let Some(error_kind) = output.payload.error_kind {
        let actual_payload = outputs[i].payload.clone();

        let diag = Some(vec![format!(
          "Expected an {} error kind, but payload was: {:?}",
          error_kind, actual_payload
        )]);

        test_block.add_test(
          move || match actual_payload {
            MessageTransport::Failure(Failure::Exception(_)) => (error_kind == "Exception"),
            MessageTransport::Failure(Failure::Error(_)) => (error_kind == "Error"),
            _ => false,
          },
          "Error kind matches",
          diag,
        );
      }
      if let Some(error_msg) = output.payload.error_msg {
        let actual_payload = outputs[i].payload.clone();

        let diag = Some(vec![format!(
          "Expected error message '{}', but payload was: {:?}",
          error_msg, actual_payload
        )]);

        test_block.add_test(
          move || match actual_payload {
            MessageTransport::Failure(Failure::Exception(msg)) => (error_msg == msg),
            MessageTransport::Failure(Failure::Error(msg)) => (error_msg == msg),
            _ => false,
          },
          "Error message matches",
          diag,
        );
      }
    }
    harness.add_block(test_block);
  }
  harness.print();

  Ok(())
}

fn make_diagnostic<T: std::fmt::Debug, U: std::fmt::Debug>(
  actual: &T,
  expected: &U,
) -> Option<Vec<String>> {
  Some(vec![
    format!("Actual: {:?}", actual),
    format!("Expected: {:?}", expected),
  ])
}
