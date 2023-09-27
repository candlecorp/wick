use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use structured_output::StructuredOutput;
use tracing::Span;
use wasmtime::component::{Component, Linker};
use wasmtime::Store;
use wasmtime_wasi::preview2::{self, Table, WasiCtxBuilder};
use wick_config::config::{AppConfiguration, BoundIdentifier, TriggerDefinition};
use wick_runtime::Runtime;
use wick_trigger::resources::Resource;
use wick_trigger::Trigger;

use crate::state::{self, SimpleState};
use crate::Error;

#[derive(Default, Clone, Copy)]
#[allow(missing_debug_implementations)]
#[non_exhaustive]
pub struct WasmTrigger {}

#[async_trait]
impl Trigger for WasmTrigger {
  async fn run(
    &self,
    _name: String,
    _runtime: Runtime,
    _app_config: AppConfiguration,
    config: TriggerDefinition,
    _resources: Arc<HashMap<BoundIdentifier, Resource>>,
    parent_span: Span,
  ) -> Result<StructuredOutput, wick_trigger::Error> {
    let TriggerDefinition::WasmCommand(config) = config else {
      panic!("invalid trigger definition, expected WasmCommand configuraton");
    };
    let span = info_span!("trigger:wasm-command");
    span.follows_from(parent_span);
    span.in_scope(|| {
      debug!("config: {:?}", config);
    });

    let engine = wick_wasm_engine::wasm_engine();

    let module_bytes = config
      .reference()
      .bytes(&Default::default())
      .await
      .map_err(|e| Error::ComponentFetch(Box::new(e)))?;
    let component = Component::from_binary(engine, &module_bytes).map_err(Error::ComponentLoad)?;

    let mut linker = Linker::<SimpleState>::new(engine);

    wasmtime_wasi::preview2::command::add_to_linker(&mut linker).map_err(Error::Linker)?;

    let mut table = Table::new();

    let mut wasi = WasiCtxBuilder::new();
    wasi.inherit_stdio();
    let wasi = wasi.build(&mut table).map_err(Error::WasiBuild)?;
    let mut store = Store::new(engine, SimpleState { wasi, table });

    state::generated::add_to_linker(&mut linker, |state: &mut SimpleState| state).map_err(Error::Linker)?;

    let (_bindings, instance) = state::generated::CommandTrigger::instantiate_async(&mut store, &component, &linker)
      .await
      .map_err(Error::Instantiation)?;

    let cmd = preview2::command::Command::new(&mut store, &instance).map_err(Error::WasiCommand)?;

    let exit = cmd
      .wasi_cli_run()
      .call_run(&mut store)
      .await
      .map_err(Error::CommandRun)?
      .map_or(false, |_| true);

    Ok(StructuredOutput::new("", serde_json::json!({ "success": exit })))
  }

  async fn shutdown_gracefully(self) -> Result<(), wick_trigger::Error> {
    Ok(())
  }

  async fn wait_for_done(&self) -> StructuredOutput {
    StructuredOutput::default()
  }
}

impl fmt::Display for WasmTrigger {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}
