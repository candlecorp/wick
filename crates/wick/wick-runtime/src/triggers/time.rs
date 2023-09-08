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

use super::{ComponentId, Trigger, TriggerKind};
use crate::dev::prelude::*;
use crate::resources::Resource;
use crate::Runtime;

async fn invoke_operation(
  runtime: Runtime,
  target: Entity,
  payload: Arc<Vec<config::OperationInputConfig>>,
  span: &Span,
) -> Result<(), RuntimeError> {
  let packets: Vec<_> = payload
    .iter()
    .map(|packet| Packet::encode(packet.name(), packet.value()))
    .collect();

  let invocation = Invocation::new(
    Entity::server("schedule_client"),
    target,
    packets,
    InherentData::unsafe_default(),
    span,
  );

  let mut response = runtime.invoke(invocation, Default::default()).await?;
  while let Some(packet) = response.next().await {
    trace!(?packet, "trigger:time:response");
  }
  Ok(())
}

async fn create_schedule(
  runtime: Runtime,
  schedule: Schedule,
  config: TimeTriggerConfig,
  component_id: String,
  span: Span,
) -> Result<tokio::task::JoinHandle<()>, RuntimeError> {
  // Create a scheduler loop
  let handle = tokio::spawn(async move {
    let operation = Arc::new(config.operation().name().to_owned());
    let payload = Arc::new(config.payload().to_vec());

    let mut current_count: u16 = 0;

    let (failure_tx, mut failure_rx) = tokio::sync::mpsc::channel::<()>(1);

    loop {
      if config.schedule().repeat() > 0 && current_count >= config.schedule().repeat() {
        break;
      }

      if failure_rx.try_recv().is_ok() {
        span.in_scope(|| error!("scheduler task failed to execute"));
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

      let target = Entity::operation(&component_id, &*operation);
      let job_span = info_span!("trigger:schedule:job", target = ?target);
      job_span.follows_from(&span);
      let rt = runtime.clone();
      let payload = payload.clone();

      let fail_tx = failure_tx.clone();

      tokio::spawn(async move {
        let fut = invoke_operation(rt, target, payload, &job_span);
        if let Err(e) = fut.await {
          job_span.in_scope(|| error!("error invoking operation: {}", e));
          let _ = fail_tx.send(()).await;
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
}

#[async_trait]
impl Trigger for Time {
  async fn run(
    &self,
    _name: String,
    runtime: Runtime,
    _app_config: AppConfiguration,
    config: TriggerDefinition,
    _resources: Arc<HashMap<String, Resource>>,
    _span: Span,
  ) -> Result<StructuredOutput, RuntimeError> {
    let TriggerDefinition::Time(config) = config else {
      return Err(RuntimeError::TriggerKind(Context::Trigger, TriggerKind::Time));
    };

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

    let span = info_span!("trigger:schedule", schedule = cron);
    let component_id = config.operation().component_id()?.to_owned();

    let scheduler_task = create_schedule(runtime, schedule, config, component_id, span).await?;

    self.handler.lock().replace(scheduler_task);

    Ok(StructuredOutput::new(
      "Scheduler started",
      json!({"scheduler": "started"}),
    ))
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    Ok(())
  }

  async fn wait_for_done(&self) -> StructuredOutput {
    let Some(handler) = self.handler.lock().take() else {
      return StructuredOutput::new("scheduled job never ran", json!({"status": "schedule job never ran"}));
    };

    match handler.await {
      Ok(_) => {
        info!("time trigger done");
        StructuredOutput::new("scheduled job complete", json!({"status": "schedule job complete"}))
      }
      Err(e) => {
        error!(err=%e,"time trigger error");
        let message = format!("time trigger error: {}", e);
        let json = json!({"error": message});
        StructuredOutput::new(message, json)
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

  use std::path::Path;

  use anyhow::Result;

  use super::*;
  use crate::build_trigger_runtime;
  use crate::test::load_example;

  fn remove_test_file(test_file: &Path) {
    match std::fs::remove_file(test_file) {
      Ok(_) => {
        println!("removed file {}", test_file.to_string_lossy());
      }
      Err(_) => {
        println!("no test file {}", test_file.to_string_lossy());
      }
    }
  }

  #[test_logger::test(tokio::test)]
  async fn test_time_example() -> Result<()> {
    let pwd = std::env::current_dir()?;
    std::env::set_var("ROOT_DIR", pwd.to_string_lossy().to_string());

    let app_config = load_example("time/time.wick").await?.try_app_config()?;
    let rt = build_trigger_runtime(&app_config, Span::current())?.build(None).await?;

    let pwd = std::env::current_dir()?;
    println!("pwd: {}", pwd.to_string_lossy());
    let test_file = pwd.join("time-trigger.txt");
    remove_test_file(&test_file);

    let trigger = Time::load()?;
    let trigger_config = app_config.triggers()[0].clone();
    trigger
      .run(
        "test".to_owned(),
        rt,
        app_config,
        trigger_config,
        Default::default(),
        Span::current(),
      )
      .await?;
    debug!("scheduler is running");
    trigger.wait_for_done().await;
    let contents = std::fs::read_to_string(&test_file)?;
    assert_eq!(contents, "Running");
    remove_test_file(&test_file);

    Ok(())
  }
}
