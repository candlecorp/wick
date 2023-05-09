use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use tokio::task::{JoinError, JoinHandle};
use wick_config::config::{AppConfiguration, WickConfiguration};
use wick_runtime::error::RuntimeError;
use wick_runtime::resources::Resource;
use wick_runtime::Trigger;

use crate::{Error, Result};

/// A Wick Host wraps a Wick runtime with server functionality like persistence,.
#[must_use]
pub struct AppHost {
  manifest: AppConfiguration,
  triggers: Option<TriggerState>,
}

impl std::fmt::Debug for AppHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppHost").field("manifest", &self.manifest).finish()
  }
}

impl AppHost {
  pub fn start(&mut self, _seed: Option<u64>) -> Result<()> {
    debug!("host starting");

    let resources = self.init_resources()?;
    self.start_triggers(resources)?;

    Ok(())
  }

  /// Stops a running host.
  #[allow(clippy::unused_async)]
  pub async fn stop(self) {
    debug!("host stopping");
  }

  fn init_resources(&mut self) -> Result<HashMap<String, Resource>> {
    let mut resources = HashMap::new();
    for (id, def) in self.manifest.resources() {
      let resource = Resource::new(def.kind.clone())?;
      resources.insert(id.clone(), resource);
    }
    Ok(resources)
  }

  fn start_triggers(&mut self, resources: HashMap<String, Resource>) -> Result<()> {
    assert!(self.triggers.is_none(), "triggers already started");
    let resources = Arc::new(resources);
    let mut triggers = TriggerState::new();
    for trigger_config in self.manifest.triggers() {
      debug!(?trigger_config, "loading trigger");
      let config = trigger_config.clone();
      let name = self.manifest.name().clone();
      let app_config = self.manifest.clone();

      match wick_runtime::get_trigger_loader(&trigger_config.kind()) {
        Some(loader) => {
          let loader = loader()?;
          let inner = loader.clone();
          let resources = resources.clone();
          let task = tokio::spawn(async move {
            if let Err(e) = inner.run(name, app_config, config, resources).await {
              error!("trigger failed to start: {}", e);
            };
            Ok(())
          });
          triggers.add((loader, task));
        }
        _ => {
          return Err(Error::RuntimeError(Box::new(
            wick_runtime::error::RuntimeError::InitializationFailed(format!(
              "could not find trigger {}",
              &trigger_config.kind()
            )),
          )))
        }
      };
    }
    self.triggers.replace(triggers);

    Ok(())
  }

  #[allow(clippy::unused_async)]
  pub async fn wait_for_done(&mut self) -> Result<()> {
    let state = self.triggers.take().unwrap();
    let (triggers, start_tasks): (Vec<_>, Vec<_>) = state
      .triggers
      .into_iter()
      .map(|(trigger, task)| (trigger, task.unwrap()))
      .unzip();
    join_all(start_tasks).await;
    debug!("all triggers started");
    for trigger in triggers.iter() {
      trigger.wait_for_done().await;
    }
    debug!("all triggers finished");

    Ok(())
  }
}

type SharedTrigger = Arc<dyn Trigger + Send + Sync + 'static>;
type TriggerTask = JoinHandle<std::result::Result<(), RuntimeError>>;

#[derive(Default)]
#[must_use]
#[allow(missing_debug_implementations)]
pub struct TriggerState {
  triggers: Vec<(SharedTrigger, Option<TriggerTask>)>,
}

impl TriggerState {
  pub fn new() -> Self {
    Self { triggers: vec![] }
  }

  pub fn add(
    &mut self,
    handle: (
      Arc<dyn Trigger + Send + Sync>,
      JoinHandle<std::result::Result<(), RuntimeError>>,
    ),
  ) {
    self.triggers.push((handle.0, Some(handle.1)));
  }

  pub async fn until_done(
    mut self,
  ) -> (
    std::result::Result<std::result::Result<(), RuntimeError>, JoinError>,
    usize,
    Vec<JoinHandle<std::result::Result<(), RuntimeError>>>,
  ) {
    futures::future::select_all(self.triggers.iter_mut().map(|v| v.1.take().unwrap())).await
  }
}

/// The HostBuilder builds the configuration for a Wick Host.
#[must_use]
#[derive(Debug, Clone)]
pub struct AppHostBuilder {
  manifest: AppConfiguration,
}

impl Default for AppHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl AppHostBuilder {
  /// Creates a new host builder.
  pub fn new() -> AppHostBuilder {
    AppHostBuilder {
      manifest: AppConfiguration::default(),
    }
  }

  pub async fn from_manifest_url(location: &str, allow_latest: bool, insecure_registries: &[String]) -> Result<Self> {
    let fetch_options = wick_config::config::FetchOptions::new()
      .allow_latest(allow_latest)
      .allow_insecure(insecure_registries);

    let manifest = WickConfiguration::fetch(location, fetch_options)
      .await?
      .try_app_config()?;
    Ok(Self::from_definition(manifest))
  }

  pub fn from_definition(definition: AppConfiguration) -> Self {
    AppHostBuilder { manifest: definition }
  }

  /// Constructs an instance of a Wick host.
  pub fn build(self) -> AppHost {
    AppHost {
      manifest: self.manifest,
      triggers: None,
    }
  }
}

#[cfg(test)]
mod test {

  use super::*;

  #[test]
  fn builds_default() {
    let _h = AppHostBuilder::new().build();
  }
}
