use std::path::PathBuf;

use structopt::StructOpt;
use tokio_stream::StreamExt;
use vino_provider::native::prelude::{
  Entity,
  TransportMap,
};
use vino_provider_cli::LoggingOptions;
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::MessageTransport;

use crate::error::VowError;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[structopt(flatten)]
  pub(crate) logging: LoggingOptions,

  #[structopt(flatten)]
  pub(crate) pull: super::PullOptions,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  /// The path to the data file.
  data_path: PathBuf,
}
#[allow(clippy::future_not_send)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  vino_provider_cli::init_logging(&opts.logging)?;

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::try_from_module(&component, 1)?;

  let data = vino_test::read_data(opts.data_path).map_err(|e| VowError::Other(e.to_string()))?;

  let mut harness = TestHarness::new();
  for test in data {
    let mut payload = TransportMap::new();
    let entity = Entity::component_direct(test.component.clone());
    for (k, v) in &test.inputs {
      payload.insert(k, MessageTransport::Success(v.clone()));
    }

    let stream = provider
      .invoke(entity, payload)
      .await
      .map_err(VowError::ComponentPanic)?;
    let outputs: Vec<_> = stream.collect().await;

    println!("# Component '{}'", test.component);

    for (i, output) in test.outputs.iter().enumerate() {
      let result = outputs[i].port == output.port;
      harness.register(move || result, "port name".to_owned());
    }
  }
  harness.run();

  Ok(())
}

struct TestHarness {
  tests: Vec<TestCase>,
}

impl TestHarness {
  pub(crate) fn new() -> Self {
    TestHarness { tests: vec![] }
  }
  pub(crate) fn register(&mut self, test: impl FnOnce() -> bool + 'static, description: String) {
    self.tests.push(TestCase {
      test: Box::new(test),
      description,
    });
  }
  pub(crate) fn run(self) {
    println!("1..{}", self.tests.len());
    for (i, test) in self.tests.into_iter().enumerate() {
      let desc = test.description.clone();
      if test.exec() {
        println!("ok {} Component '{}': port name", i, desc);
      } else {
        println!("not ok {} {} port name", i, desc);
      }
    }
  }
}

struct TestCase {
  test: Box<dyn FnOnce() -> bool>,
  description: String,
}

impl TestCase {
  pub(crate) fn exec(self) -> bool {
    (self.test)()
  }
}
