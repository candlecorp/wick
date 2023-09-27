use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use wasmtime::component::Component;
use wasmtime::Error;

static MODULE_CACHE: once_cell::sync::Lazy<Mutex<HashMap<PathBuf, Component>>> =
  once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn get_cached_component(path: &Path) -> Option<Component> {
  let cache = MODULE_CACHE.lock();
  cache.get(path).cloned()
}

pub async fn fetch_component(path: &Path) -> Result<Component, Error> {
  if let Some(lock) = get_cached_component(path) {
    return Ok(lock);
  }

  let module_bytes = tokio::fs::read(path.clone()).await?;

  let component = Component::from_binary(crate::wasm_engine(), &module_bytes)?;

  let mut cache = MODULE_CACHE.lock();
  cache.insert(path.to_path_buf(), component.clone());
  Ok(component)
}
