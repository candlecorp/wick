use std::sync::Arc;

use wick_config::config::TriggerKind;
use wick_trigger::Trigger;

use crate::error::HostError;

pub fn load_trigger(name: &TriggerKind) -> Result<Arc<dyn Trigger + Send + Sync>, HostError> {
  match name {
    TriggerKind::Cli => Ok(Arc::new(wick_trigger_cli::Cli::default())),
    TriggerKind::Http => Ok(Arc::new(wick_trigger_http::Http::default())),
    TriggerKind::Time => Ok(Arc::new(wick_trigger_time::Time::default())),
    TriggerKind::WasmCommand => Ok(Arc::new(wick_trigger_wasm_command::WasmTrigger::default())),
  }
}
