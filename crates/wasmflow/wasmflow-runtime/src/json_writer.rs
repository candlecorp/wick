use std::path::PathBuf;

use parking_lot::Mutex;
use wasmflow_interpreter::{EventKind, Observer};

#[derive(Default)]
pub(crate) struct JsonWriter {
  path: PathBuf,
  events: Mutex<Vec<serde_json::Value>>,
  states: Mutex<Vec<serde_json::Value>>,
}

impl JsonWriter {
  pub(crate) fn new(path: PathBuf) -> Self {
    Self {
      path,
      ..Default::default()
    }
  }
}

impl Observer for JsonWriter {
  fn on_event(&self, index: usize, event: &wasmflow_interpreter::Event) {
    let tx_id = event.tx_id();
    let entry = match event.kind() {
      EventKind::Ping(_) => serde_json::Value::Null,
      EventKind::TransactionStart(tx) => {
        serde_json::json!({
    "type":event.name(),
    "index": index,
    "tx_id": tx_id.to_string(),
    "name" : tx.schematic_name()})
      }
      EventKind::TransactionDone => {
        serde_json::json!({
          "type":event.name(),
          "index": index,
          "tx_id": tx_id.to_string()
        })
      }
      EventKind::CallComplete(data) => {
        serde_json::json!({
    "type":event.name(),
    "index": index,
    "tx_id": tx_id.to_string(),
    "component_index":data.index()})
      }
      EventKind::Invocation(component_index, _invocation) => {
        serde_json::json!({
    "type":event.name(),
    "index": index,
    "tx_id": tx_id.to_string(),
    "invocation": "temp",
    "component_index":component_index})
      }
      EventKind::PortData(port) => {
        serde_json::json!({
          "type":event.name(),
          "index": index,
          "tx_id": tx_id.to_string(),
          "dir":port.direction().to_string(),
          "port_index":port.port_index(),
          "component_index":port.node_index()
        })
      }
      EventKind::PortStatusChange(port) => {
        serde_json::json!({
          "type":event.name(),
          "index": index,
          "tx_id": tx_id.to_string(),
          "dir":port.direction().to_string(),
          "port_index":port.port_index(),
          "component_index":port.node_index()
        })
      }
      EventKind::Close(error) => {
        serde_json::json!({
          "type":event.name(),
          "index": index,
          "error": error.as_ref().map(|e|e.to_string()).unwrap_or_default()
        })
      }
    };
    let mut lock = self.events.lock();
    lock.push(entry);
  }

  fn on_after_event(&self, _index: usize, state: &wasmflow_interpreter::State) {
    // let state = serde_json::Value::Array(state.json_transactions());
    // let mut lock = self.states.lock();
    // lock.push(state);
  }

  fn on_close(&self) {
    let mut list = Vec::new();
    let events = self.events.lock();
    let states = self.states.lock();
    for (i, event) in events.iter().enumerate() {
      let mut map = serde_json::Map::new();
      map.insert("event".to_owned(), event.clone());
      map.insert("state".to_owned(), states.get(i).cloned().unwrap_or_default());
      list.push(serde_json::Value::Object(map));
    }
    let json = serde_json::Value::Array(list);
    let js = format!("{}", json);
    info!("writing {}", self.path.to_string_lossy());
    std::fs::write(&self.path, js).unwrap();
  }
}
