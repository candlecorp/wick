pub(crate) mod state;

use std::time::Duration;

use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tracing_futures::Instrument;

use super::channel::{Event, EventKind, InterpreterChannel, InterpreterDispatchChannel};
use super::error::Error;
use super::InterpreterOptions;
use crate::interpreter::event_loop::state::State;
use crate::interpreter::executor::error::ExecutionError;

#[derive(Debug)]
pub(crate) struct EventLoop {
  channel: Option<InterpreterChannel>,
  dispatcher: InterpreterDispatchChannel,
  task: Mutex<Option<JoinHandle<Result<(), ExecutionError>>>>,
}

impl EventLoop {
  pub(crate) const WAKE_TIMEOUT: Duration = Duration::from_millis(500);
  pub(crate) const HUNG_TX_TIMEOUT: Duration = Duration::from_millis(2000);

  pub(super) fn new(channel: InterpreterChannel) -> Self {
    let dispatcher = channel.dispatcher();
    Self {
      channel: Some(channel),
      dispatcher,
      task: Mutex::new(None),
    }
  }

  pub(super) async fn start(&mut self, options: InterpreterOptions, observer: Option<Box<dyn Observer + Send + Sync>>) {
    trace!("starting event loop");
    let channel = self.channel.take().unwrap();

    let span = trace_span!("event_loop");

    let handle = tokio::spawn(async move { event_loop(channel, options, observer).instrument(span).await });
    let mut lock = self.task.lock();
    lock.replace(handle);
  }

  fn steal_task(&self) -> Option<JoinHandle<Result<(), ExecutionError>>> {
    let mut lock = self.task.lock();
    lock.take()
  }

  pub(super) async fn shutdown(&self) -> Result<(), Error> {
    trace!("shutting down event loop");
    let task = self.steal_task();
    match task {
      Some(task) => {
        self.dispatcher.dispatch_close(None).await;
        trace!("yielding to loop");
        let timeout = std::time::Duration::from_secs(2);
        let result = tokio::time::timeout(timeout, task).await;
        result
          .map_err(|e| Error::Shutdown(e.to_string()))?
          .map_err(|e| Error::Shutdown(e.to_string()))??;
        debug!("event loop closed");
      }
      None => {
        warn!("shutdown called but no task running");
      }
    }

    Ok(())
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    trace!("dropping event loop");
    let lock = self.task.lock();
    if let Some(task) = &*lock {
      task.abort();
    }
  }
}

pub trait Observer {
  fn on_event(&self, index: usize, event: &Event);
  fn on_after_event(&self, index: usize, state: &State);
  fn on_close(&self);
}

async fn event_loop(
  mut channel: InterpreterChannel,
  options: InterpreterOptions,
  observer: Option<Box<dyn Observer + Send + Sync>>,
) -> Result<(), ExecutionError> {
  debug!(?options, "started");
  let mut state = State::new(channel.dispatcher());

  let mut num: usize = 0;

  let result = loop {
    let task = tokio::time::timeout(EventLoop::WAKE_TIMEOUT, channel.accept());
    match task.await {
      Ok(Some(event)) => {
        let tx_id = event.tx_id;

        if let Some(observer) = &observer {
          observer.on_event(num, &event);
        }

        let span = trace_span!("event", otel.name = event.name(), i = num, %tx_id);
        debug!(event = ?event, tx_id = ?tx_id, "iteration");
        let name = event.name().to_owned();

        let result = match event.kind {
          EventKind::Invocation(_index, _invocation) => {
            error!("invocation not supported");
            panic!("invocation not supported")
          }
          EventKind::CallComplete(data) => state.handle_call_complete(tx_id, data).instrument(span).await,
          EventKind::PortData(data) => state.handle_port_data(tx_id, data).instrument(span).await,
          EventKind::TransactionDone => state.handle_transaction_done(tx_id).instrument(span).await,
          EventKind::TransactionStart(transaction) => {
            state.handle_transaction_start(*transaction).instrument(span).await
          }
          EventKind::Ping(ping) => {
            trace!(ping);
            Ok(())
          }
          EventKind::Close(error) => match error {
            Some(error) => {
              error!(%error,"stopped with error");
              break Err(error);
            }
            None => {
              debug!("stopping");
              break Ok(());
            }
          },
        };

        if let Err(e) = result {
          if options.error_on_missing && matches!(e, ExecutionError::MissingTx(_)) {
            error!(event = %name, tx_id = ?tx_id, response_error = %e, "iteration:end");
            channel.dispatcher().dispatch_close(Some(e)).await;
          } else {
            warn!(event = %name, tx_id = ?tx_id, response_error = %e, "iteration:end");
          }
        } else {
          trace!(event = %name, tx_id = ?tx_id, "iteration:end");
        }

        if let Some(observer) = &observer {
          observer.on_after_event(num, &state);
        }
        num += 1;
      }
      Ok(None) => {
        trace!("done");
        break Ok(());
      }
      Err(_) => {
        if let Err(error) = state.check_hung(options.error_on_hung).await {
          error!(%error,"Error checking hung transactions");
          channel.dispatcher().dispatch_close(Some(error)).await;
        };
      }
    }
  };
  trace!("stopped");
  if let Some(observer) = &observer {
    observer.on_close();
  }
  result
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum EventLoopError {
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
