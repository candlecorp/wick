pub(super) mod generated {
  #![allow(clippy::future_not_send, clippy::impl_trait_in_params)]
  use wasmtime::component::bindgen;
  bindgen!({
    world: "component",
    path: "../../../wit/",
    async:true,
  });
  pub(crate) use candle::wick::wick::*;
}

#[async_trait::async_trait]
#[allow(unused)]
impl generated::Host for super::state::ComponentState {
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
