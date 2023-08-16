use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use config::{AppConfiguration, TimeTriggerConfig, TriggerDefinition};
use cron::Schedule;
use parking_lot::Mutex;
use serde_json::json;
use structured_output::StructuredOutput;
use tokio::time::Duration;
use tokio_stream::StreamExt;
use tracing::Span;
use wick_packet::{Entity, InherentData, Packet};

use super::{build_trigger_runtime, Trigger, TriggerKind};
use crate::dev::prelude::*;
use crate::resources::Resource;
use crate::triggers::resolve_ref;

async fn invoke_operation(
  runtime: Arc<crate::Runtime>,
  operation: Arc<String>,
  payload: Arc<Vec<config::OperationInputConfig>>,
) -> Result<(), RuntimeError> {
  let packets: Vec<_> = payload
    .iter()
    .map(|packet| Packet::encode(packet.name(), packet.value()))
    .collect();

  let invocation = Invocation::new(
    Entity::server("schedule_client"),
    Entity::operation("0", operation.as_str()),
    packets,
    InherentData::unsafe_default(),
    &Span::current(),
  );

  let mut response = runtime.invoke(invocation, Default::default()).await?;
  while let Some(packet) = response.next().await {
    trace!(?packet, "trigger:time:response");
  }
  Ok(())
}

async fn create_schedule(
  schedule: Schedule,
  app_config: AppConfiguration,
  config: TimeTriggerConfig,
) -> Result<tokio::task::JoinHandle<()>, RuntimeError> {
  let span = info_span!("trigger:schedule", schedule = ?schedule);
  let mut runtime = build_trigger_runtime(&app_config, span.clone())?;
  match resolve_ref(&app_config, config.operation().component())? {
    super::ResolvedComponent::Ref(_, _) => {}
    super::ResolvedComponent::Inline(def) => {
      let schedule_binding = config::ImportBinding::component("0", def.clone());
      runtime.add_import(schedule_binding);
    }
  };

  let runtime = runtime.build(None).await?;

  // Create a scheduler loop
  let handle = tokio::spawn(async move {
    let runtime = Arc::new(runtime);
    let operation = Arc::new(config.operation().name().to_owned());
    let payload = Arc::new(config.payload().to_vec());

    let mut current_count: u16 = 0;

    loop {
      if config.schedule().repeat() > 0 && current_count >= config.schedule().repeat() {
        break;
      }

      current_count += 1;

      // Calculate the next scheduled time based on the current time
      let next = schedule.upcoming(Utc).next().unwrap();

      // Calculate the duration until the next scheduled time
      let duration = next.signed_duration_since(Utc::now());
      span.in_scope(|| debug!("duration until next schedule: {:?}", duration));

      tokio::time::sleep(Duration::from_millis(duration.num_milliseconds() as u64)).await;

      span.in_scope(|| debug!("done sleeping"));

      let rt_clone = runtime.clone();
      let operation_clone = operation.clone();
      let payload_clone = payload.clone();

      let fut = invoke_operation(rt_clone, operation_clone, payload_clone);
      let span = span.clone();
      tokio::spawn(async move {
        if let Err(e) = fut.await {
          span.in_scope(|| error!("Error invoking operation: {}", e));
        }
      });
    }
  });
  Ok(handle)
}

#[derive(Debug)]
pub(crate) struct Time {
  #[allow(dead_code)]
  name: String,
  handler: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Time {
  pub(crate) fn new() -> Self {
    Self {
      name: "Schedule".to_owned(),
      handler: Default::default(),
    }
  }

  pub(crate) fn load() -> Result<Arc<dyn Trigger + Send + Sync>, RuntimeError> {
    Ok(Arc::new(Self::new()))
  }

  async fn handle(
    &self,
    app_config: AppConfiguration,
    config: TimeTriggerConfig,
  ) -> Result<StructuredOutput, RuntimeError> {
    let cron = config.schedule().cron().to_owned();

    // Create a new cron schedule that runs every minute
    let schedule = Schedule::from_str(&cron);
    let schedule = match schedule {
      Ok(schedule) => schedule,
      Err(e) => {
        return Err(RuntimeError::ScheduleStartError(format!(
          "Unable to create schedule from cron expression: {}",
          e
        )))
      }
    };

    let scheduler_task = create_schedule(schedule, app_config, config).await?;

    self.handler.lock().replace(scheduler_task);

    Ok(StructuredOutput::new(
      "Scheduler started",
      json!({"scheduler": "started"}),
    ))
  }
}

#[async_trait]
impl Trigger for Time {
  async fn run(
    &self,
    _name: String,
    app_config: AppConfiguration,
    config: TriggerDefinition,
    _resources: Arc<HashMap<String, Resource>>,
    _span: Span,
  ) -> Result<StructuredOutput, RuntimeError> {
    let config = if let TriggerDefinition::Time(config) = config {
      config
    } else {
      return Err(RuntimeError::InvalidConfig(Context::Trigger, TriggerKind::Time));
    };

    self.handle(app_config, config).await
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    Ok(())
  }

  async fn wait_for_done(&self) {
    let Some(handler) = self.handler.lock().take() else {
      return;
    };

    match handler.await {
      Ok(_) => {
        info!("cron done");
      }
      Err(e) => {
        error!("cron error: {}", e);
      }
    }
  }
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Time Trigger")
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use anyhow::Result;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_basic() -> Result<()> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_dir = crate_dir.join("../../../examples/time/");

    let yaml = manifest_dir.join("time.wick");
    let app_config = config::WickConfiguration::fetch(&yaml, Default::default())
      .await?
      .finish()?
      .try_app_config()?;

    let trigger = Time::load()?;
    let trigger_config = app_config.triggers()[0].clone();
    trigger
      .run(
        "test".to_owned(),
        app_config,
        trigger_config,
        Default::default(),
        Span::current(),
      )
      .await?;
    debug!("scheduler is running");
    trigger.wait_for_done().await;

    Ok(())
  }
}
