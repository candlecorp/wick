pub(crate) mod channel;
pub(crate) mod collections;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;

use std::sync::Arc;

use seeded_random::{Random, Seed};
use tracing_futures::Instrument;
use wick_interface_types::CollectionSignature;
use wick_packet::{Entity, Invocation, PacketStream};

use self::channel::InterpreterDispatchChannel;
use self::collections::HandlerMap;
use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use crate::constants::*;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::collections::collection_collection::CollectionCollection;
use crate::interpreter::collections::schematic_collection::SchematicCollection;
use crate::interpreter::executor::error::ExecutionError;
use crate::{Collection, NamespaceHandler, Observer};

#[must_use]
#[derive()]
pub struct Interpreter {
  rng: Random,
  program: Program,
  event_loop: EventLoop,
  signature: CollectionSignature,
  collections: Arc<HandlerMap>,
  self_collection: Arc<SchematicCollection>,
  dispatcher: InterpreterDispatchChannel,
  namespace: Option<String>,
}

impl std::fmt::Debug for Interpreter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Interpreter")
      .field("program", &self.program)
      .field("event_loop", &self.event_loop)
      .field("signature", &self.signature)
      .field("collections", &self.collections)
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
    collections: Option<HandlerMap>,
  ) -> Result<Self, Error> {
    debug!("init");
    let rng = seed.map_or_else(Random::new, Random::from_seed);
    let mut handlers = collections.unwrap_or_default();
    handlers.add_core(&network);

    // Add the collection:: collection
    let collection_collection = CollectionCollection::new(&handlers);
    handlers.add(NamespaceHandler {
      namespace: NS_COLLECTIONS.to_owned(),
      collection: Arc::new(Box::new(collection_collection)),
    });

    let signatures = handlers.collection_signatures();

    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();

    // Make the self:: collection
    let collections = Arc::new(handlers);
    let self_collection = SchematicCollection::new(collections.clone(), program.state(), &dispatcher, rng.seed());
    let self_signature = self_collection.list().clone();

    debug!(?self_signature, "signature");

    let event_loop = EventLoop::new(channel);
    debug!(
      schematics = ?program.schematics().iter().map(|s| s.name()).collect::<Vec<_>>(),
      "schematics handled by this interpreter"
    );

    Ok(Self {
      rng,
      program,
      dispatcher,
      signature: self_signature,
      collections,
      self_collection,
      event_loop,
      namespace,
    })
  }

  async fn invoke_schematic(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream, Error> {
    let dispatcher = self.dispatcher.clone();
    let name = invocation.target.name().to_owned();
    let schematic = self
      .program
      .schematics()
      .iter()
      .find(|s| s.name() == name)
      .ok_or_else(|| {
        Error::SchematicNotFound(
          invocation.target.clone(),
          self.program.schematics().iter().map(|s| s.name().to_owned()).collect(),
        )
      })?;

    let executor = SchematicExecutor::new(schematic.clone(), dispatcher.clone());
    Ok(
      executor
        .invoke(
          invocation,
          stream,
          self.rng.seed(),
          self.collections.clone(),
          self.self_collection.clone(),
        )
        .instrument(tracing::span::Span::current())
        .await?,
    )
  }

  pub async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream, Error> {
    let known_targets = || {
      let mut hosted: Vec<_> = self.collections.collections().keys().cloned().collect();
      if let Some(ns) = &self.namespace {
        hosted.push(ns.clone());
      }
      hosted
    };
    let span = debug_span!("invoke");

    let stream = match &invocation.target {
      Entity::Operation(ns, _) => {
        if ns == NS_SELF || ns == Entity::LOCAL || Some(ns) == self.namespace.as_ref() {
          self.invoke_schematic(invocation, stream).instrument(span).await?
        } else {
          trace!(?invocation);
          self
            .collections
            .get(ns)
            .ok_or_else(|| Error::TargetNotFound(invocation.target.clone(), known_targets()))?
            .collection
            .handle(invocation, stream, None)
            .instrument(span)
            .await
            .map_err(ExecutionError::CollectionError)?
        }
      }
      _ => return Err(Error::TargetNotFound(invocation.target, known_targets())),
    };

    Ok(stream)
  }

  pub fn get_export_signature(&self) -> &CollectionSignature {
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
    for (ns, collection) in self.collections.collections() {
      debug!(namespace = %ns, "shutting down collection");
      if let Err(error) = collection
        .collection
        .shutdown()
        .await
        .map_err(|e| Error::CollectionShutdown(e.to_string()))
      {
        warn!(%error,"error during shutdown");
      };
    }

    shutdown
  }
}

#[derive(Default, Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct InterpreterOptions {
  /// Stop the interpreter and return an error on any hung transactions.
  pub error_on_hung: bool,
  /// Stop the interpreter and return an error if any messages come after a transaction has completed.
  pub error_on_missing: bool,
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
