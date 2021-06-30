use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use vino_rpc::port::{
  Port,
  PortStream,
  Sender,
};

pub(crate) struct OutputSender {
  port: Arc<Mutex<Port>>,
}
impl OutputSender {
  fn new(name: String) -> Self {
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

#[derive(Default)]
pub(crate) struct InvocationMap {
  outputs: Vec<String>,
  map: HashMap<String, HashMap<String, OutputSender>>,
}

impl InvocationMap {
  pub(crate) fn new(outputs: Vec<String>) -> Self {
    Self {
      outputs,
      ..InvocationMap::default()
    }
  }

  pub(crate) fn get(&self, id: &str) -> Option<&HashMap<String, OutputSender>> {
    self.map.get(id)
  }

  pub(crate) fn new_invocation(&mut self, inv_id: String) -> PortStream {
    let (tx, rx) = self.make_channel();
    self.map.insert(inv_id, tx);
    rx
  }

  pub(crate) fn make_channel(&self) -> (HashMap<String, OutputSender>, PortStream) {
    let outputs: HashMap<String, OutputSender> = self
      .outputs
      .iter()
      .map(|name| (name.clone(), OutputSender::new(name.clone())))
      .collect();
    let ports = outputs.iter().map(|(_, o)| o.port.clone()).collect();
    let receiver = PortStream::new(ports);
    (outputs, receiver)
  }
}
