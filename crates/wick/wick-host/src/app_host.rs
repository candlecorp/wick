use std::collections::HashMap;
use std::sync::Arc;

use futures::future::{join_all, select};
use futures::pin_mut;
use structured_output::StructuredOutput;
use tokio::task::{JoinError, JoinHandle};
use tracing::Span;
use wick_config::config::{AppConfiguration, BoundIdentifier};
use wick_config::WickConfiguration;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, InherentData, Invocation, PacketStream, RuntimeConfig};
use wick_runtime::error::RuntimeError;
use wick_runtime::Runtime;
use wick_trigger::resources::Resource;
use wick_trigger::{build_trigger_runtime, Trigger};

use crate::triggers::load_trigger;
use crate::Result;

#[derive(derive_builder::Builder)]
#[builder(derive(Debug), setter(into))]
/// A Wick Host wraps a Wick runtime with server functionality like persistence,.
#[must_use]
pub struct AppHost {
  manifest: AppConfiguration,
  #[builder(setter(skip))]
  triggers: Option<TriggerState>,
  runtime: Runtime,
  #[builder(default = "tracing::Span::current()")]
  span: Span,
}

impl std::fmt::Debug for AppHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppHost").field("manifest", &self.manifest).finish()
  }
}

impl AppHost {
  pub fn start(&mut self) -> Result<()> {
    self.span.in_scope(|| debug!("host starting"));

    let resources = self.init_resources()?;
    self.start_triggers(resources)?;

    Ok(())
  }

  /// Stops a running host.
  #[allow(clippy::unused_async)]
  pub async fn stop(self) {
    self.span.in_scope(|| debug!("host stopping"));
  }

  fn init_resources(&mut self) -> Result<HashMap<BoundIdentifier, Resource>> {
    let mut resources = HashMap::new();
    for def in self.manifest.resources() {
      let resource = Resource::new(def.kind().clone())?;
      resources.insert(def.binding().clone(), resource);
    }
    Ok(resources)
  }

  pub async fn build_runtime(config: &AppConfiguration, seed: Option<u64>, span: Span) -> Result<Runtime> {
    let rt = build_trigger_runtime(config, span)
      .unwrap()
      .build(seed.map(seeded_random::Seed::unsafe_new))
      .await?;
    Ok(rt)
  }

  fn start_triggers(&mut self, resources: HashMap<BoundIdentifier, Resource>) -> Result<()> {
    assert!(self.triggers.is_none(), "triggers already started");

    let resources = Arc::new(resources);
    let mut triggers = TriggerState::new();

    for trigger_config in self.manifest.triggers() {
      self.span.in_scope(|| debug!(?trigger_config, "loading trigger"));
      let config = trigger_config.clone();
      let name = self.manifest.name().to_owned();
      let app_config = self.manifest.clone();
      let trigger = load_trigger(&trigger_config.kind())?;

      let inner = trigger.clone();
      let resources = resources.clone();
      let span = info_span!("trigger", kind=%trigger_config.kind());
      span.follows_from(&self.span);
      let rt = self.runtime.clone();

      let task = tokio::spawn(async move {
        span.in_scope(|| trace!("initializing trigger"));
        match inner.run(name, rt, app_config, config, resources, span.clone()).await {
          Ok(_output) => {
            span.in_scope(|| debug!("trigger initialized"));
          }
          Err(e) => {
            span.in_scope(|| error!("trigger failed to start: {}", e));
          }
        }
        Ok(())
      });
      triggers.add((trigger, task));
    }
    self.triggers.replace(triggers);

    Ok(())
  }

  pub async fn request(
    &self,
    target: Entity,
    config: Option<RuntimeConfig>,
    stream: PacketStream,
    data: InherentData,
  ) -> Result<PacketStream> {
    let invocation = Invocation::new(Entity::server(Entity::LOCAL), target, stream, data, &self.span);
    Ok(self.runtime.invoke(invocation, config).await?)
  }

  #[allow(clippy::unused_async)]
  pub async fn wait_for_done(&mut self) -> Result<Vec<StructuredOutput>> {
    let state = self.triggers.take().unwrap();
    let (triggers, start_tasks): (Vec<_>, Vec<_>) = state
      .triggers
      .into_iter()
      .map(|(trigger, task)| (trigger, task.unwrap()))
      .unzip();
    join_all(start_tasks).await;
    self.span.in_scope(|| debug!("all triggers started"));
    let mut all_output = Vec::new();
    for trigger in &triggers {
      let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
      };
      pin_mut!(ctrl_c);
      match select(ctrl_c, trigger.wait_for_done()).await {
        futures::future::Either::Left(_) => {
          self.span.in_scope(|| debug!("ctrl-c received, stopping triggers"));
          break;
        }
        futures::future::Either::Right((output, _)) => {
          self.span.in_scope(|| debug!("trigger finished"));
          all_output.push(output);
        }
      }
    }
    self.span.in_scope(|| debug!("all triggers finished"));

    Ok(all_output)
  }
}

#[async_trait::async_trait]
impl crate::Host for AppHost {
  fn namespace(&self) -> &str {
    Entity::LOCAL
  }

  fn get_signature(&self, path: Option<&[&str]>, entity: Option<&Entity>) -> Result<ComponentSignature> {
    Ok(self.runtime.deep_signature(path, entity)?)
  }

  async fn invoke(&self, invocation: Invocation, data: Option<RuntimeConfig>) -> Result<PacketStream> {
    Ok(self.runtime.invoke_deep(None, invocation, data).await?)
  }

  async fn invoke_deep(
    &self,
    path: Option<&[&str]>,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
  ) -> Result<PacketStream> {
    Ok(self.runtime.invoke_deep(path, invocation, data).await?)
  }

  fn get_active_config(&self) -> WickConfiguration {
    WickConfiguration::App(self.manifest.clone())
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

impl AppHostBuilder {
  /// Creates a new host builder.
  #[must_use]
  pub fn new() -> AppHostBuilder {
    AppHostBuilder::default()
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
