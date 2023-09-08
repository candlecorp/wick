pub(crate) mod state;

use std::time::Duration;

use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tracing::Span;
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
  span: Span,
}

impl EventLoop {
  pub(crate) const WAKE_TIMEOUT: Duration = Duration::from_millis(500);
  pub(crate) const STALLED_TX_TIMEOUT: Duration = Duration::from_secs(60 * 5);
  pub(crate) const SLOW_TX_TIMEOUT: Duration = Duration::from_secs(15);

  pub(super) fn new(channel: InterpreterChannel, span: &Span) -> Self {
    let event_span = info_span!("event_loop");
    event_span.follows_from(span);
    let dispatcher = channel.dispatcher(Some(event_span.clone()));

    Self {
      channel: Some(channel),
      dispatcher,
      task: Mutex::new(None),
      span: event_span,
    }
  }

  pub(super) async fn start(&mut self, options: InterpreterOptions, observer: Option<Box<dyn Observer + Send + Sync>>) {
    let channel = self.channel.take().unwrap();

    let span = self.span.clone();
    let handle = tokio::spawn(async move { event_loop(channel, options, observer, span).await });
    let mut lock = self.task.lock();
    lock.replace(handle);
  }

  fn steal_task(&self) -> Option<JoinHandle<Result<(), ExecutionError>>> {
    let mut lock = self.task.lock();
    lock.take()
  }

  pub(super) async fn shutdown(&self) -> Result<(), Error> {
    self.span.in_scope(|| trace!("shutting down event loop"));
    let task = self.steal_task();
    match task {
      Some(task) => {
        self.dispatcher.dispatch_close(None);

        let timeout = std::time::Duration::from_secs(2);
        let result = tokio::time::timeout(timeout, task).await;
        match result.map_err(|_| Error::ShutdownTimeout)? {
          Ok(Err(e)) => {
            return Err(Error::Shutdown(e.to_string()));
          }
          Err(e) => {
            self.span.in_scope(|| warn!(%e, "event loop panicked"));
            return Err(Error::EventLoopPanic(e.to_string()));
          }
          Ok(_) => {}
        };
        self.span.in_scope(|| debug!("event loop closed"));
      }
      None => {
        self.span.in_scope(|| warn!("shutdown called but no task running"));
      }
    }

    Ok(())
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    self.span.in_scope(|| trace!("dropping event loop"));
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
  span: Span,
) -> Result<(), ExecutionError> {
  debug!(?options, "started");
  let mut state = State::new(channel.dispatcher(None));

  let mut num: usize = 0;

  let result = loop {
    let task = tokio::time::timeout(EventLoop::WAKE_TIMEOUT, channel.accept());
    match task.await {
      Ok(Some(event)) => {
        let ctx_id = event.ctx_id;

        if let Some(observer) = &observer {
          observer.on_event(num, &event);
        }

        let name = event.name().to_owned();
        let tx_span = event.span.unwrap_or_else(Span::current);

        tx_span.in_scope(|| debug!(event = ?event.kind, ctx_id = ?ctx_id));

        let result = match event.kind {
          EventKind::Invocation(_index, _invocation) => {
            error!("invocation not supported");
            panic!("invocation not supported")
          }
          EventKind::CallComplete(data) => state.handle_call_complete(ctx_id, data).instrument(tx_span).await,
          EventKind::PortData(data) => state.handle_port_data(ctx_id, data, &tx_span).await,
          EventKind::ExecutionDone => state.handle_exec_done(ctx_id).instrument(tx_span).await,
          EventKind::ExecutionStart(context) => state.handle_exec_start(*context, &options).instrument(tx_span).await,
          EventKind::Ping(ping) => {
            trace!(ping);
            Ok(())
          }
          EventKind::Close(error) => match error {
            Some(error) => {
              tx_span.in_scope(|| error!(%error,"stopped with error"));
              break Err(error);
            }
            None => {
              tx_span.in_scope(|| debug!("stopping"));
              break Ok(());
            }
          },
        };

        span.in_scope(|| {
          if let Err(e) = result {
            warn!(event = %name, ctx_id = ?ctx_id, response_error = %e, "iteration:end");
          } else {
            trace!(event = %name, ctx_id = ?ctx_id, "iteration:end");
          }
        });

        if let Some(observer) = &observer {
          observer.on_after_event(num, &state);
        }
        num += 1;
      }
      Ok(None) => {
        break Ok(());
      }
      Err(_) => {
        span.in_scope(|| {
          if let Err(error) = state.run_cleanup() {
            error!(%error,"Error checking hung invocations");
            channel.dispatcher(None).dispatch_close(Some(error));
          };
        });
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

  const fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  const fn test_sync_send() -> Result<()> {
    sync_send::<EventLoop>();
    Ok(())
  }
}
