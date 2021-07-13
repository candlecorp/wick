use std::sync::{
  Arc,
  Mutex,
};

use vino_rpc::port::{
  Port,
  Sender,
};

#[derive(Debug)]
pub(crate) struct OutputSender {
  pub(crate) port: Arc<Mutex<Port>>,
}
impl OutputSender {
  pub(crate) fn new(name: String) -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new(name))),
    }
  }
}

impl Sender for OutputSender {
  type PayloadType = Vec<u8>;
  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}
