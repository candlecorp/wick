#[allow(unused)]
pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
pub mod types {
  #[allow(unused)]
  use super::types;
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct Interactive {
    pub stdin: bool,
    pub stdout: bool,
    pub stderr: bool,
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {}
