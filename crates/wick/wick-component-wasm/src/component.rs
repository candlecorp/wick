mod bindgen;
mod setup;
mod state;

use std::fmt;

use flow_component::Component as FlowComponent;
use state::ComponentState;
use tracing::Span;
use wasmtime::component::{Instance, Linker};
use wasmtime::Store;
use wasmtime_wasi::preview2::Table;
use wick_config::FetchableAssetReference;
use wick_interface_types::ComponentSignature;

use self::setup::ComponentSetup;
pub use self::setup::{ComponentSetupBuilder, ComponentSetupBuilderError};
use crate::wasi::{compute_preopen_dirs, init_ctx};
use crate::Error;

#[derive()]
#[allow(missing_debug_implementations)]
#[non_exhaustive]
pub struct WasmComponent {
  #[allow(unused)]
  bindings: bindgen::generated::Component,
  #[allow(unused)]
  instance: Instance,
}

impl FlowComponent for WasmComponent {
  fn handle(
    &self,
    _invocation: wick_packet::Invocation,
    _data: Option<wick_packet::RuntimeConfig>,
    _callback: flow_component::LocalScope,
  ) -> flow_component::BoxFuture<Result<wick_packet::PacketStream, flow_component::ComponentError>> {
    todo!()
  }

  fn signature(&self) -> &ComponentSignature {
    todo!()
  }
}

impl WasmComponent {
  pub async fn try_load(
    ns: &str,
    asset: FetchableAssetReference<'_>,
    options: ComponentSetup,
    span: Span,
  ) -> Result<Self, Error> {
    let location = asset.location();
    span.in_scope(|| trace!("loading wasm component from {} for ns {}", location, ns));

    let mut table = Table::new();
    let perms = options.permissions.unwrap_or_default();

    let preopen_dirs = compute_preopen_dirs(perms.dirs().iter())?;
    let wasi = init_ctx(&mut table, preopen_dirs, &[], &[])?;

    let path = asset.path().map_err(|e| Error::Asset(Box::new(e)))?;

    let component = wick_wasm_engine::store::fetch_component(&path)
      .await
      .map_err(Error::ComponentFetch)?;
    let engine = wick_wasm_engine::wasm_engine();

    let mut linker = Linker::<ComponentState>::new(engine);

    wasmtime_wasi::preview2::command::add_to_linker(&mut linker).map_err(Error::Linker)?;

    let mut store = Store::new(engine, ComponentState::new(wasi, table, options.callback));

    bindgen::generated::add_to_linker(&mut linker, |state: &mut ComponentState| state).map_err(Error::Linker)?;
    let fut = bindgen::generated::Component::instantiate_async(&mut store, &component, &linker);
    // bindgen::generated::add_to_linker(&mut linker, |state: &mut ComponentState| state).map_err(Error::Linker)?;
    // let fut = bindgen::generated::Component::instantiate_async(&mut store, lock.component(), &linker);

    let (bindings, instance) = fut.await.map_err(Error::Instantiation)?;

    // let cmd = preview2::command::Command::new(&mut store, &instance).map_err(Error::WasiCommand)?;

    // let exit = cmd
    //   .wasi_cli_run()
    //   .call_run(&mut store)
    //   .await
    //   .map_err(Error::CommandRun)?
    //   .map_or(false, |_| true);

    Ok(Self { bindings, instance })
  }
}

impl fmt::Display for WasmComponent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}
