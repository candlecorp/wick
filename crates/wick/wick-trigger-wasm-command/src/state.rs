use wasmtime_wasi::preview2::{Table, WasiCtx, WasiView};

pub(super) mod generated {
  #![allow(clippy::future_not_send, clippy::impl_trait_in_params)]
  use wasmtime::component::bindgen;
  bindgen!({
    world: "command-trigger",
    path: "../../../wit/",
    async:true,
  });
  pub(crate) use candle::wick::wick::*;
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

#[async_trait::async_trait]
#[allow(unused)]
impl generated::Host for SimpleState {
  async fn request_sync(
    &mut self,
    invocation: generated::Invocation,
  ) -> wasmtime::Result<Result<generated::Response, generated::InvocationError>> {
    todo!()
  }

  async fn request_async(&mut self, invocation: generated::Invocation) -> wasmtime::Result<u64> {
    todo!()
  }

  async fn get_response(
    &mut self,
    id: u64,
  ) -> wasmtime::Result<Result<generated::Response, generated::InvocationError>> {
    todo!()
  }

  async fn cancel_request(&mut self, id: u64) -> wasmtime::Result<()> {
    todo!()
  }
}
