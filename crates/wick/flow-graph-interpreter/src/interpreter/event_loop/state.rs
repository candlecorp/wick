use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use flow_graph::{PortDirection, PortReference};
use uuid::Uuid;
use wick_packet::PacketPayload;

use super::EventLoop;
use crate::interpreter::channel::{CallComplete, InterpreterDispatchChannel};
use crate::interpreter::executor::context::{ExecutionContext, TxState};
use crate::interpreter::executor::error::ExecutionError;
use crate::InterpreterOptions;

#[derive(Debug)]
pub struct State {
  context_map: ContextMap,
  channel: InterpreterDispatchChannel,
}

impl State {
  pub(super) fn new(channel: InterpreterDispatchChannel) -> Self {
    Self {
      context_map: ContextMap::default(),
      channel,
    }
  }

  fn get_tx(&self, uuid: &Uuid) -> Option<&(ExecutionContext, Metadata)> {
    self.context_map.get(uuid)
  }

  pub fn invocations(&self) -> &ContextMap {
    &self.context_map
  }

  pub(super) fn run_cleanup(&mut self) -> Result<(), ExecutionError> {
    let mut cleanup = Vec::new();
    for (id, (tx, meta)) in self.context_map.iter() {
      let last_update = tx.last_access().elapsed().unwrap();

      let active_instances = tx.active_instances().iter().map(|i| i.id()).collect::<Vec<_>>();
      if last_update > EventLoop::SLOW_TX_TIMEOUT {
        if active_instances.is_empty() && tx.done() {
          cleanup.push(*id);

          continue;
        }

        if !meta.have_warned() {
          warn!(%id, ?active_instances, "slow tx: no packet received in a long time");
          meta.set_have_warned();
        }
      }
      if last_update > EventLoop::STALLED_TX_TIMEOUT {
        match tx.check_stalled() {
          Ok(TxState::Finished) => {
            // execution has completed its output and isn't generating more data, clean it up.
            cleanup.push(*id);
          }
          Ok(TxState::OutputPending) => {
            error!(%id, "tx reached timeout while still waiting for output data");
            cleanup.push(*id);
          }
          Ok(TxState::CompleteWithTasksPending) => {
            error!(%id, "tx reached timeout while still waiting for tasks to complete");
            cleanup.push(*id);
          }
          Err(error) => {
            error!(%error, %id, "stalled invocation generated error determining hung state");
          }
        }
      }
    }
    for ctx_id in cleanup {
      self.context_map.remove(&ctx_id);
    }
    Ok(())
  }

  fn get_mut(&mut self, ctx_id: &Uuid) -> Option<&mut ExecutionContext> {
    self.context_map.get_mut(ctx_id)
  }

  pub(super) async fn handle_exec_start(
    &mut self,
    mut ctx: ExecutionContext,
    options: &InterpreterOptions,
  ) -> Result<(), ExecutionError> {
    match ctx.start(options).await {
      Ok(_) => {
        self.context_map.init_tx(ctx.id(), ctx);
        trace!("execution started");
        Ok(())
      }
      Err(e) => Err(e),
    }
  }

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_exec_done(&mut self, ctx_id: Uuid) -> Result<(), ExecutionError> {
    let is_done = if let Some(tx) = self.get_mut(&ctx_id) {
      let statistics = tx.finish()?;
      trace!(?statistics);
      tx.active_instances().is_empty()
    } else {
      false
    };
    if is_done {
      debug!(%ctx_id,"execution done");
      self.context_map.remove(&ctx_id);
    }
    Ok(())
  }

  #[allow(clippy::unused_async)]
  async fn handle_input_data(&mut self, ctx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    let (tx, _) = match self.get_tx(&ctx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          port = %port, %ctx_id, "still receiving upstream data for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };

    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);
    let instance = tx.instance(port.node_index());

    tx.stats
      .mark(format!("input:{}:{}:ready", port.node_index(), port.port_index()));

    let is_schematic_output = port.node_index() == graph.output().index();

    if is_schematic_output {
      debug!(
        operation = %instance,
        port = port_name,
        "handling schematic output"
      );

      tx.handle_schematic_output()?;
    } else if let Some(packet) = tx.take_instance_input(&port) {
      if packet.is_error() {
        warn!(
          operation = %instance,
          port = port_name,
          payload = ?packet,
          "handling port input"
        );
      } else {
        debug!(
          operation = %instance,
          port = port_name,
          payload = ?packet,
          "handling port input"
        );
      }

      tx.push_packets(port.node_index(), vec![packet]).await?;
    }
    Ok(())
  }

  #[allow(clippy::unused_async)]
  async fn handle_output_data(&mut self, ctx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    let (tx, _) = match self.get_tx(&ctx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          port = %port, %ctx_id, "still receiving downstream data for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };

    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    let instance = tx.instance(port.node_index());

    tx.stats
      .mark(format!("output:{}:{}:ready", port.node_index(), port.port_index()));

    if let Some(packet) = tx.take_instance_output(&port) {
      if packet.is_error() {
        warn!(
          operation = %instance,
          port = port_name,
          payload = ?packet,
          "handling port output"
        );
      } else {
        debug!(
          operation = %instance,
          port = port_name,
          payload = ?packet,
          "handling port output"
        );
      }
      let connections = graph.get_port(&port).connections();
      for index in connections {
        let connection = &graph.connections()[*index];
        let downport = *connection.to();
        let name = graph.get_port_name(&downport);

        let channel = self.channel.clone();
        let downstream_instance = tx.instance(downport.node_index()).clone();
        let message = packet.clone().set_port(name);
        trace!(%connection, "delivering packet to downstream",);
        downstream_instance.buffer_in(&downport, message);
        channel.dispatch_data(ctx_id, downport);
      }
    } else {
      panic!("got port_data message with no payload to act on, port: {:?}", port);
    }

    Ok(())
  }

  pub(super) async fn handle_port_data(&mut self, ctx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    match port.direction() {
      PortDirection::Out => self.handle_output_data(ctx_id, port).await,
      PortDirection::In => self.handle_input_data(ctx_id, port).await,
    }
  }

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_call_complete(&self, ctx_id: Uuid, data: CallComplete) -> Result<(), ExecutionError> {
    let (tx, _) = match self.get_tx(&ctx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          ?data,
          %ctx_id, "tried to cleanup call for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };
    let instance = tx.instance(data.index);
    debug!(operation = instance.id(), entity = %instance.entity(), "call complete");

    if let Some(PacketPayload::Err(err)) = data.err {
      warn!(?err, "op:error");
      // If the call contains an error, then the component panicked.
      // We need to propagate the error downward...
      tx.handle_op_err(data.index, &err)?;
      // ...and clean up the call.
      // instance.handle_stream_complete(CompletionStatus::Error)?;
    }

    Ok(())
  }
}

#[derive(Debug)]
struct Metadata {
  have_warned_long_tx: AtomicBool,
}

impl Default for Metadata {
  fn default() -> Self {
    Self {
      have_warned_long_tx: AtomicBool::new(false),
    }
  }
}

impl Metadata {
  fn have_warned(&self) -> bool {
    self.have_warned_long_tx.load(std::sync::atomic::Ordering::Relaxed)
  }

  fn set_have_warned(&self) {
    self
      .have_warned_long_tx
      .store(true, std::sync::atomic::Ordering::Relaxed);
  }
}

#[derive(Debug, Default)]
#[must_use]
pub struct ContextMap(HashMap<Uuid, (ExecutionContext, Metadata)>);

impl ContextMap {
  pub(crate) fn init_tx(&mut self, uuid: Uuid, ctx: ExecutionContext) {
    self.0.insert(uuid, (ctx, Metadata::default()));
  }

  fn get(&self, uuid: &Uuid) -> Option<&(ExecutionContext, Metadata)> {
    self.0.get(uuid).map(|tx| {
      tx.0.update_last_access();
      tx
    })
  }

  fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut ExecutionContext> {
    self.0.get_mut(uuid).map(|tx| {
      tx.0.update_last_access();
      &mut tx.0
    })
  }

  fn remove(&mut self, uuid: &Uuid) -> Option<(ExecutionContext, Metadata)> {
    self.0.remove(uuid)
  }

  fn iter(&self) -> impl Iterator<Item = (&Uuid, &(ExecutionContext, Metadata))> {
    self.0.iter()
  }
}
