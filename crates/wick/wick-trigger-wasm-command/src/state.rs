use wasmtime_wasi::preview2::{Table, WasiCtx, WasiView};

pub(super) mod generated {
  #![allow(clippy::future_not_send, clippy::impl_trait_in_params)]
  use wasmtime::component::bindgen;
  bindgen!({
    world: "command-trigger",
    async:true,
  });
}

pub(super) struct SimpleState {
  pub(super) wasi: WasiCtx,
  pub(super) table: Table,
}

impl WasiView for SimpleState {
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
