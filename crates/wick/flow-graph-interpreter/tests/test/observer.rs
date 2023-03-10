use flow_graph_interpreter::{EventKind, Observer};
use parking_lot::Mutex;

#[derive(Default)]
pub struct JsonWriter {
  events: Mutex<Vec<serde_json::Value>>,
  states: Mutex<Vec<serde_json::Value>>,
}

impl Observer for JsonWriter {
  fn on_event(&self, index: usize, event: &flow_graph_interpreter::Event) {
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

  fn on_after_event(&self, _index: usize, _state: &flow_graph_interpreter::State) {
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
    println!("writing event_loop.json");
    std::fs::write("event_loop.json", js).unwrap();
  }
}
