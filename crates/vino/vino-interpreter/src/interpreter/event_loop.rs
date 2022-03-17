mod state;

use tokio::task::JoinHandle;
use tracing::Level;
use tracing_futures::Instrument;

use super::channel::InterpreterChannel;
use super::error::Error;
use crate::interpreter::channel::{Event, EventKind};
use crate::interpreter::event_loop::state::State;
use crate::interpreter::executor::error::ExecutionError;
use crate::InterpreterDispatchChannel;

#[derive(Debug)]
pub(super) struct EventLoop {
  channel: Option<InterpreterChannel>,
  dispatcher: InterpreterDispatchChannel,
  task: Option<JoinHandle<()>>,
}
impl EventLoop {
  pub(super) fn new(channel: InterpreterChannel) -> Self {
    let dispatcher = channel.dispatcher();
    Self {
      channel: Some(channel),
      dispatcher,
      task: None,
    }
  }

  #[instrument(skip(self), name = "event_loop")]
  pub(super) async fn start(&mut self) {
    trace!("starting");
    let channel = self.channel.take().unwrap();

    let handle = tokio::spawn(async move {
      event_loop(channel).await;
    });

    self.task = Some(handle);
  }

  #[instrument(skip(self), name = "event_loop")]
  pub(super) async fn shutdown(mut self) -> Result<(), Error> {
    let task = self.task.take();
    match task {
      Some(task) => {
        let _ = self.dispatcher.dispatch(Event::close()).await?;
        trace!("aborting task");
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), task)
          .await
          .map_err(|e| Error::ShutdownFailed(e.to_string()))?;
        debug!("shutdown complete");
      }
      None => {
        warn!("shutdown called but no task running");
      }
    }

    Ok(())
  }
}

impl Drop for EventLoop {
  #[instrument(skip(self), name = "event_loop:drop")]
  fn drop(&mut self) {
    trace!("dropping event loop");
    if let Some(task) = &self.task {
      task.abort();
    }
  }
}

#[instrument(skip(channel), name = "event_loop")]
async fn event_loop(mut channel: InterpreterChannel) {
  trace!("started");
  let mut state = State::new();

  #[cfg(test)]
  let mut events = Vec::new();
  let mut num: usize = 0;

  loop {
    match channel.accept().await {
      Some(event) => {
        let tx_id = event.tx_id;
        #[cfg(test)]
        let mut debug_entry = {
          let entry = match &event.kind {
            EventKind::Ping(_) => serde_json::Value::Null,
            EventKind::TransactionStart(tx) => {
              serde_json::json!({"type":event.name(), "index": num, "tx_id": tx.id().to_string(), "name" : tx.schematic_name()})
            }
            EventKind::TransactionDone => {
              serde_json::json!({"type":event.name(), "index": num, "tx_id": tx_id.to_string()})
            }
            EventKind::CallComplete(data) => {
              serde_json::json!({"type":event.name(), "index": num, "tx_id": tx_id.to_string(), "index":data.index})
            }
            EventKind::PortData(port) => {
              serde_json::json!({
                "type":event.name(),
                "tx_id": tx_id.to_string(),
                "dir":port.direction().to_string(),
                "port_index":port.port_index(),
                "component_index":port.component_index()
              })
            }
            EventKind::PortStatusChange(port) => {
              serde_json::json!({
                "type":event.name(),
                "tx_id": tx_id.to_string(),
                "dir":port.direction().to_string(),
                "port_index":port.port_index(),
                "component_index":port.component_index()
              })
            }
            EventKind::Close => {
              serde_json::json!({"type":event.name()})
            }
          };
          let mut map = serde_json::Map::new();
          map.insert("event".to_owned(), entry);
          map
        };

        let span = span!(Level::TRACE, "event", event = event.name(), i = num, ?tx_id);

        let result = match event.kind {
          EventKind::CallComplete(data) => state.handle_call_complete(tx_id, data).instrument(span).await,
          EventKind::PortData(data) => state.handle_port_data(tx_id, data).instrument(span).await,
          EventKind::PortStatusChange(port) => state.handle_port_status_change(tx_id, port).instrument(span).await,
          EventKind::TransactionDone => state.handle_transaction_done(tx_id).instrument(span).await,
          EventKind::TransactionStart(transaction) => {
            state.handle_transaction_start(*transaction).instrument(span).await
          }
          EventKind::Ping(ping) => {
            trace!(ping);
            Ok(())
          }
          EventKind::Close => {
            debug!("stopping");
            break;
          }
        };

        if let Err(e) = result {
          warn!(response_error = ?e);
        }

        #[cfg(test)]
        {
          debug_entry.insert(
            "schematics".to_owned(),
            serde_json::Value::Array(state.debug_print_transactions()),
          );
          events.push(serde_json::Value::Object(debug_entry));
        }
        num += 1;
      }
      None => {
        trace!("done");
        break;
      }
    }
  }
  trace!("stopped");
  #[cfg(test)]
  {
    let json = serde_json::Value::Array(events);
    let js = format!("{}", json);
    std::fs::write("event_loop.json", js).unwrap();
  }
}

#[derive(thiserror::Error, Debug)]
pub enum EventLoopError {
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),
  #[error(transparent)]
  ChannelError(#[from] crate::interpreter::channel::Error),
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test_logger::test]
  fn test_sync_send() -> Result<()> {
    sync_send::<EventLoop>();
    Ok(())
  }
}
