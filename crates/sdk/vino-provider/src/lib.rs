use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_rpc::port::Receiver;
pub mod error;
pub mod provider_macro;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = error::ProviderError;
pub type Context<T> = Arc<Mutex<T>>;

#[async_trait]
pub trait VinoProviderComponent {
  type Context;
  fn get_name(&self) -> String;
  fn get_input_ports(&self) -> Vec<(String, String)>;
  fn get_output_ports(&self) -> Vec<(String, String)>;
  async fn job_wrapper(
    &self,
    context: Arc<Mutex<Self::Context>>,
    data: HashMap<String, Vec<u8>>,
  ) -> std::result::Result<Receiver, Box<dyn std::error::Error + Send + Sync>>;
}

pub use vino_rpc::Component;
