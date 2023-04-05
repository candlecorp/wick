pub(crate) mod channel;
pub(crate) mod components;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use seeded_random::{Random, Seed};
use tracing_futures::Instrument;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, Invocation, PacketStream};

use self::channel::InterpreterDispatchChannel;
use self::components::HandlerMap;
use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use crate::constants::*;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::components::component_component::ComponentComponent;
use crate::interpreter::components::schematic_component::SchematicComponent;
use crate::interpreter::executor::error::ExecutionError;
use crate::{Component, NamespaceHandler, Observer, SharedHandler};

#[must_use]
#[derive()]
pub struct Interpreter {
  rng: Random,
  program: Program,
  event_loop: EventLoop,
  signature: ComponentSignature,
  components: Arc<HandlerMap>,
  self_component: Arc<SchematicComponent>,
  dispatcher: InterpreterDispatchChannel,
  namespace: Option<String>,
  exposed_ops: HashMap<String, SharedHandler>, // A map from op name to the ns of the handler that exposes it.
}

impl std::fmt::Debug for Interpreter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Interpreter")
      .field("program", &self.program)
      .field("event_loop", &self.event_loop)
      .field("signature", &self.signature)
      .field("collections", &self.components)
      .field("dispatcher", &self.dispatcher)
      .finish()
  }
}

impl Interpreter {
  #[instrument(name="interpreter-init", skip_all, fields(namespace = %namespace.as_ref().map_or("n/a", String::as_str)))]
  pub fn new(
    seed: Option<Seed>,
    network: Network,
    namespace: Option<String>,
    components: Option<HandlerMap>,
  ) -> Result<Self, Error> {
    debug!("init");
    let rng = seed.map_or_else(Random::new, Random::from_seed);
    let mut handlers = components.unwrap_or_default();
    let mut exposed_ops = HashMap::new();
    // Create a map of operation names to the namespace of the handler.
    for handler in handlers.inner().values() {
      if handler.is_exposed() {
        for op in &handler.component.list().operations {
          exposed_ops.insert(op.name.clone(), handler.component.clone());
        }
      }
    }
    handlers.add_core(&network)?;

    // Add the component:: component
    let component_component = ComponentComponent::new(&handlers);
    handlers.add(NamespaceHandler::new(NS_COMPONENTS, Box::new(component_component)))?;

    let signatures = handlers.component_signatures();
    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();

    // Make the self:: component
    let components = Arc::new(handlers);
    let self_component = SchematicComponent::new(components.clone(), program.state(), &dispatcher, rng.seed());
    let self_signature = self_component.list().clone();

    debug!(?self_signature, "signature");

    let event_loop = EventLoop::new(channel);
    debug!(
      operations = ?program.operations().iter().map(|s| s.name()).collect::<Vec<_>>(),
      "operations handled by this interpreter"
    );

    Ok(Self {
      rng,
      program,
      dispatcher,
      signature: self_signature,
      components,
      self_component,
      event_loop,
      namespace,
      exposed_ops,
    })
  }

  async fn invoke_operation(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream, Error> {
    let dispatcher = self.dispatcher.clone();
    let name = invocation.target.name().to_owned();
    let op = self
      .program
      .operations()
      .iter()
      .find(|s| s.name() == name)
      .ok_or_else(|| {
        Error::OpNotFound(
          invocation.target.clone(),
          self.program.operations().iter().map(|s| s.name().to_owned()).collect(),
        )
      })?;

    let executor = SchematicExecutor::new(op.clone(), dispatcher.clone());
    Ok(
      executor
        .invoke(
          invocation,
          stream,
          self.rng.seed(),
          self.components.clone(),
          self.self_component.clone(),
        )
        .instrument(tracing::span::Span::current())
        .await?,
    )
  }

  pub async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream, Error> {
    let known_targets = || {
      let mut hosted: Vec<_> = self.components.inner().keys().cloned().collect();
      if let Some(ns) = &self.namespace {
        hosted.push(ns.clone());
      }
      hosted
    };
    let span = trace_span!("invoke");

    let stream = match &invocation.target {
      Entity::Operation(ns, name) => {
        if ns == NS_SELF || ns == Entity::LOCAL || Some(ns) == self.namespace.as_ref() {
          if let Some(component) = self.exposed_ops.get(name) {
            trace!(entity=%invocation.target, "invoke::exposed::operation");
            return Ok(
              component
                .handle(invocation, stream, None)
                .instrument(span)
                .await
                .map_err(ExecutionError::ComponentError)?,
            );
          }
          trace!(entity=%invocation.target, "invoke::composite::operation");
          self.invoke_operation(invocation, stream).instrument(span).await?
        } else {
          trace!(entity=%invocation.target, "invoke::instance::operation");
          self
            .components
            .get(ns)
            .ok_or_else(|| Error::TargetNotFound(invocation.target.clone(), known_targets()))?
            .component
            .handle(invocation, stream, None)
            .instrument(span)
            .await
            .map_err(ExecutionError::ComponentError)?
        }
      }
      _ => return Err(Error::TargetNotFound(invocation.target, known_targets())),
    };

    Ok(stream)
  }

  pub fn get_export_signature(&self) -> &ComponentSignature {
    &self.signature
  }

  pub async fn start(
    &mut self,
    options: Option<InterpreterOptions>,
    observer: Option<Box<dyn Observer + Send + Sync>>,
  ) {
    self.event_loop.start(options.unwrap_or_default(), observer).await;
  }

  #[instrument(skip(self))]
  pub async fn shutdown(&self) -> Result<(), Error> {
    let shutdown = self.event_loop.shutdown().await;
    if let Err(error) = &shutdown {
      error!(%error,"error shutting down event loop");
    };
    for (ns, components) in self.components.inner() {
      debug!(namespace = %ns, "shutting down collection");
      if let Err(error) = components
        .component
        .shutdown()
        .await
        .map_err(|e| Error::ComponentShutdown(e.to_string()))
      {
        warn!(%error,"error during shutdown");
      };
    }

    shutdown
  }
}

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct InterpreterOptions {
  /// Stop the interpreter and return an error on any hung transactions.
  pub error_on_hung: bool,
  /// Stop the interpreter and return an error if any messages come after a transaction has completed.
  pub error_on_missing: bool,
  /// Timeout after which a component that has received no output is considered dead.
  pub output_timeout: Duration,
  /// Timeout after which a transaction that has had no events is considered hung.
  pub hung_tx_timeout: Duration,
}

impl Default for InterpreterOptions {
  fn default() -> Self {
    Self {
      error_on_hung: false,
      error_on_missing: false,
      output_timeout: Duration::from_secs(5),
      hung_tx_timeout: Duration::from_millis(500),
    }
  }
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
    sync_send::<Interpreter>();
    Ok(())
  }
}
