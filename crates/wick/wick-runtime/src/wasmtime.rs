use once_cell::sync::Lazy;

#[allow(clippy::expect_used)]
pub(crate) static WASMTIME_ENGINE: Lazy<wasmtime::Engine> = Lazy::new(|| {
  let mut config = wasmtime::Config::default();
  config.strategy(wasmtime::Strategy::Cranelift);

  if let Err(e) = config.cache_config_load_default() {
    panic!("Could not load wasmtime cache config : {}", e);
  }

  wasmtime::Engine::new(&config).expect("Could not configure Wasmtime instance")
});
