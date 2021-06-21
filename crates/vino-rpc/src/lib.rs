pub mod component_service;
pub mod error;
pub mod generated;
pub mod port;

use async_trait::async_trait;
pub use component_service::ComponentService;
pub use generated::vino::*;

pub type Result<T> = std::result::Result<T, error::RpcError>;
pub type Error = crate::error::RpcError;
pub type RpcResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_use]
extern crate log;

#[async_trait]
pub trait RpcHandler: Send + Sync {
  async fn request(
    &self,
    inv_id: String,
    component: String,
    payload: Vec<u8>,
  ) -> std::result::Result<
    crate::port::Receiver,
    Box<dyn std::error::Error + Sync + std::marker::Send>,
  >;
}
