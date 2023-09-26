use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use wick_config::config::TriggerKind;
use wick_trigger::Trigger;

use crate::error::HostError;

pub(crate) type TriggerLoader = Arc<dyn Fn() -> Result<Arc<dyn Trigger + Send + Sync>, HostError> + Send + Sync>;

static TRIGGER_LOADER_REGISTRY: Lazy<Mutex<HashMap<TriggerKind, TriggerLoader>>> = Lazy::new(|| {
  let mut m: HashMap<TriggerKind, TriggerLoader> = HashMap::new();
  m.insert(
    TriggerKind::Cli,
    Arc::new(|| Ok(Arc::new(wick_trigger_cli::Cli::default()))),
  );
  m.insert(
    TriggerKind::Http,
    Arc::new(|| Ok(Arc::new(wick_trigger_http::Http::default()))),
  );
  m.insert(
    TriggerKind::Time,
    Arc::new(|| Ok(Arc::new(wick_trigger_time::Time::default()))),
  );
  Mutex::new(m)
});

#[must_use]
pub fn get_trigger_loader(name: &TriggerKind) -> Option<TriggerLoader> {
  TRIGGER_LOADER_REGISTRY.lock().get(name).cloned()
}
