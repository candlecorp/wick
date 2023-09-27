use flow_component::LocalScope;
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiView};

pub(super) struct ComponentState {
  pub(super) wasi: WasiCtx,
  pub(super) table: Table,
  #[allow(unused)]
  callback: LocalScope,
}

impl std::fmt::Debug for ComponentState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ComponentState").field("table", &self.table).finish()
  }
}

impl WasiView for ComponentState {
  fn table(&self) -> &Table {
    &self.table
  }

  fn table_mut(&mut self) -> &mut Table {
    &mut self.table
  }

  fn ctx(&self) -> &WasiCtx {
    &self.wasi
  }

  fn ctx_mut(&mut self) -> &mut WasiCtx {
    &mut self.wasi
  }
}

impl ComponentState {
  #[must_use]
  pub(super) const fn new(wasi: WasiCtx, table: Table, callback: LocalScope) -> Self {
    Self { wasi, table, callback }
  }
}
