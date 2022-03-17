use std::collections::HashMap;

use futures::StreamExt;
use tokio::task::JoinHandle;
use tracing::Level;
use uuid::Uuid;
use vino_schematic_graph::PortDirection;
use vino_transport::{MessageSignal, MessageTransport};

use super::channel::{ComponentReady, InterpreterChannel, PortData};
use super::error::Error;
use super::executor::transaction::Transaction;
use crate::interpreter::channel::InterpreterEvent;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::error::ExecutionError;
use crate::{InterpreterDispatchChannel, Providers};

#[derive(Debug)]
pub(super) struct EventLoop {
  channel: Option<InterpreterChannel>,
  dispatcher: InterpreterDispatchChannel,
  task: Option<JoinHandle<()>>,
  providers: Option<Providers>,
}

#[derive(Debug)]
struct State {
  channel: InterpreterDispatchChannel,
  transactions: TransactionMap,
  providers: Providers,
}

impl State {
  fn new(channel: InterpreterDispatchChannel, providers: Providers) -> Self {
    Self {
      channel,
      providers,
      transactions: TransactionMap::default(),
    }
  }

  fn get_tx(&self, uuid: &Uuid) -> Result<&Transaction, ExecutionError> {
    self.transactions.get(uuid).ok_or(ExecutionError::MissingTx(*uuid))
  }

  #[instrument(name = "tx_done", skip_all)]
  fn handle_transaction_done(&mut self, uuid: Uuid) -> Result<(), ExecutionError> {
    let tx = self.transactions.remove(&uuid).ok_or(ExecutionError::MissingTx(uuid))?;
    trace!(?uuid);
    let statistics = tx.finish()?;
    trace!(?statistics);
    Ok(())
  }

  #[instrument(name = "tx_output", skip_all)]
  async fn handle_transaction_output(&mut self, data: PortData) -> Result<(), ExecutionError> {
    let tx = self.get_tx(&data.tx_id)?;
    tx.emit_data(data.transport).await?;
    Ok(())
  }

  #[instrument(name = "port_data", skip_all)]
  async fn handle_port_incoming(&mut self, data: PortData) -> Result<(), ExecutionError> {
    trace!(port = ?data.port, name = data.transport.port.as_str());
    let tx = self.get_tx(&data.tx_id)?;
    tx.stats.mark(format!(
      "data:{}:{}:ready",
      data.port.component_index(),
      data.port.port_index()
    ));
    let graph = tx.get_graph();

    trace!(target="message",payload = ?data.transport.payload);
    match data.port.direction() {
      PortDirection::In => {
        if let MessageTransport::Signal(signal) = &data.transport.payload {
          trace!("consuming signal");
          match signal {
            MessageSignal::Done => tx.set_port_done(data.port)?,
            MessageSignal::OpenBracket => todo!(),
            MessageSignal::CloseBracket => todo!(),
          }
        } else {
          let component = tx.get_handler(data.port.component_index())?;
          component.receive(&data.port, data.transport).await?;
          if component.is_ready() {
            self
              .channel
              .dispatch(InterpreterEvent::ComponentReady(ComponentReady::new(
                data.tx_id,
                data.port.component_index(),
              )))
              .await?;
          }
        }
      }
      PortDirection::Out => {
        if let MessageTransport::Signal(signal) = &data.transport.payload {
          match signal {
            MessageSignal::Done => tx.set_port_done(data.port)?,
            MessageSignal::OpenBracket => todo!(),
            MessageSignal::CloseBracket => todo!(),
          }
        }
        if graph.output().index() == data.port.component_index() {
          trace!("emitting tx output");
          self.channel.dispatch(InterpreterEvent::TransactionOutput(data)).await?;
          let ports = graph.output().output_refs();
          let done = ports.iter().all(|p| tx.is_done_closed(p) == Ok(true));
          if done {
            trace!("done");
            self
              .channel
              .dispatch(InterpreterEvent::TransactionDone(tx.id()))
              .await?;
          }
        } else {
          let connections = graph.get_port(&data.port).connections();
          for index in connections {
            let connection = graph.connections()[*index];
            let downport = connection.to();
            let name = graph.get_port_name(downport);

            trace!(connections = connection.to_string().as_str(), "delivering packet",);
            self
              .channel
              .dispatch(InterpreterEvent::PortData(data.clone().move_port(*downport, name)))
              .await?;
          }
        }
      }
    }
    Ok(())
  }

  #[instrument(name = "component_ready", skip_all)]
  async fn handle_component_ready(&self, data: ComponentReady) -> Result<(), ExecutionError> {
    let tx = self.get_tx(&data.tx_id)?;
    tx.stats.mark(format!("component:{}:ready", data.index));
    let component = tx.get_handler(data.index)?;
    let component_name = component.name().to_owned();
    let component_ns = component.namespace().to_owned();
    let payload = component.collect_transport_map().await?;

    let provider = self
      .providers
      .get(&component_ns)
      .ok_or_else(|| ExecutionError::InvalidState(StateError::MissingProvider(component.namespace().to_owned())))?;
    let stream = provider
      .provider
      .handle(&component_name, payload)
      .await
      .map_err(ExecutionError::from)?;

    let channel = self.channel.clone();
    let tx_id = data.tx_id;
    let refmap = component.output_refmap();
    trace!("refmap: {:?}", refmap);

    let mut stream = stream.map(move |packet| {
      trace!("packet:{:?}", packet);
      PortData::new(tx_id, *refmap.get(&packet.port).unwrap(), packet)
    });

    tokio::spawn(async move {
      let span = trace_span!(
        "component:output",
        component = component_name.as_str(),
        namespace = component_ns.as_str()
      );
      let _guard = span.enter();
      trace!("starting output task");
      while let Some(packet) = stream.next().await {
        trace!("received packet for {}", packet.port);
        if let Err(e) = channel.dispatch(InterpreterEvent::PortData(packet)).await {
          error!("could not send packet: {}", e);
        };
      }
    });

    Ok(())
  }
}

#[derive(Debug, Default)]
struct TransactionMap(HashMap<Uuid, Transaction>);

impl TransactionMap {
  pub(crate) fn init_tx(&mut self, uuid: Uuid, transaction: Transaction) {
    self.0.insert(uuid, transaction);
  }
  fn get(&self, uuid: &Uuid) -> Option<&Transaction> {
    self.0.get(uuid)
  }
  fn remove(&mut self, uuid: &Uuid) -> Option<Transaction> {
    self.0.remove(uuid)
  }
}

impl EventLoop {
  pub(super) fn new(providers: Providers) -> Self {
    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();
    Self {
      channel: Some(channel),
      providers: Some(providers),
      dispatcher,
      task: None,
    }
  }

  pub(super) fn channel(&self) -> InterpreterDispatchChannel {
    self.dispatcher.clone()
  }

  #[instrument(skip(self), name = "event_loop")]
  pub(super) async fn start(&mut self) {
    trace!("starting");
    let channel = self.channel.take().unwrap();
    let providers = self.providers.take().unwrap();

    let handle = tokio::spawn(async move {
      event_loop(channel, providers).await;
    });

    self.task = Some(handle);
  }

  #[instrument(skip(self), name = "event_loop")]
  pub(super) async fn shutdown(mut self) -> Result<(), Error> {
    let task = self.task.take();
    match task {
      Some(task) => {
        let _ = self.dispatcher.dispatch(InterpreterEvent::Close).await?;
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

#[instrument(skip(channel, providers), name = "event_loop")]
async fn event_loop(mut channel: InterpreterChannel, providers: Providers) {
  trace!("started");
  let mut state = State::new(channel.dispatcher(), providers);

  loop {
    match channel.accept().await {
      Ok(Some(event)) => {
        let span = span!(Level::TRACE, "event", kind = event.name());
        let _guard = span.enter();
        let result = match event {
          InterpreterEvent::TransactionOutput(data) => {
            trace!(
              tx_id = ?data.tx_id,
              component = data.port.component_index(),
              port = data.port.port_index(),
            );
            state.handle_transaction_output(data).await
          }
          InterpreterEvent::Ping(ping) => {
            trace!(ping);
            Ok(())
          }
          InterpreterEvent::ComponentReady(data) => {
            trace!(tx_id = data.tx_id.to_string().as_str(), component = &data.index,);
            state.handle_component_ready(data).await
          }
          InterpreterEvent::PortData(data) => {
            trace!(
              tx_id = ?data.tx_id,
              component = data.port.component_index(),
              port = data.port.port_index()
            );
            state.handle_port_incoming(data).await
          }
          InterpreterEvent::TransactionDone(tx_id) => {
            trace!(?tx_id);
            state.handle_transaction_done(tx_id)
          }
          InterpreterEvent::TransactionStart(mut transaction) => {
            trace!(target:"tx_start", tx_id = transaction.id().to_string().as_str());

            match transaction.start().await {
              Ok(_) => {
                state.transactions.init_tx(transaction.id(), *transaction);
                Ok(())
              }
              Err(e) => {
                error!(tx_error = ?e);
                Err(e)
              }
            }
          }
          InterpreterEvent::Close => {
            break;
          }
        };
        if let Err(e) = result {
          warn!(response_error = ?e);
        }
      }
      Ok(None) => {
        trace!("done");
        break;
      }
      Err(e) => {
        warn!(event_error = ?e);
        break;
      }
    }
  }
  trace!("stopping");
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
