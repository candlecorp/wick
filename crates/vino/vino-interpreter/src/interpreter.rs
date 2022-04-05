pub(crate) mod channel;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;
pub(crate) mod provider;

use std::sync::Arc;

use tracing_futures::Instrument;
use vino_entity::Entity;
use vino_random::{Random, Seed};
use vino_transport::{Invocation, TransportStream};
use vino_types::ProviderSignature;

use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use self::provider::HandlerMap;
use crate::constants::*;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::provider::provider_provider::ProviderProvider;
use crate::interpreter::provider::schematic_provider::SchematicProvider;
use crate::{ExecutionError, InterpreterDispatchChannel, Observer, Provider, ProviderNamespace};

#[must_use]
#[derive()]
pub struct Interpreter {
  rng: Random,
  program: Program,
  event_loop: EventLoop,
  signature: ProviderSignature,
  providers: Arc<HandlerMap>,
  self_provider: Arc<SchematicProvider>,
  dispatcher: InterpreterDispatchChannel,
  namespace: Option<String>,
}

impl std::fmt::Debug for Interpreter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Interpreter")
      .field("program", &self.program)
      .field("event_loop", &self.event_loop)
      .field("signature", &self.signature)
      .field("providers", &self.providers)
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
    providers: Option<HandlerMap>,
  ) -> Result<Self, Error> {
    debug!("init");
    let rng = seed.map_or_else(Random::new, Random::from_seed);
    let mut providers = providers.unwrap_or_default();

    // Add the provider:: provider
    let provider_provider = ProviderProvider::new(&providers);
    providers.add(ProviderNamespace {
      namespace: NS_PROVIDERS.to_owned(),
      provider: Arc::new(Box::new(provider_provider)),
    });

    let signatures = providers.provider_signatures();

    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();

    // Make the self:: provider
    let providers = Arc::new(providers);
    let self_provider = SchematicProvider::new(providers.clone(), program.state(), &dispatcher, rng.seed());
    let self_signature = self_provider.list().clone();

    debug!(?self_signature, "signature");

    let event_loop = EventLoop::new(channel);

    let interpreter = Self {
      rng,
      program,
      dispatcher,
      signature: self_signature,
      providers,
      self_provider,
      event_loop,
      namespace,
    };

    Ok(interpreter)
  }

  async fn invoke_schematic(&self, invocation: Invocation) -> Result<TransportStream, Error> {
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
          self.rng.seed(),
          self.providers.clone(),
          self.self_provider.clone(),
        )
        .await?,
    )
  }

  pub async fn invoke(&self, invocation: Invocation) -> Result<TransportStream, Error> {
    let known_targets = || {
      let mut hosted: Vec<_> = self.providers.providers().keys().cloned().collect();
      if let Some(ns) = &self.namespace {
        hosted.push(ns.clone());
      }
      hosted
    };
    let stream = match &invocation.target {
      Entity::Schematic(_) => self.invoke_schematic(invocation).await?,
      Entity::Component(ns, _) => {
        if ns == NS_SELF || ns == Entity::LOCAL || Some(ns) == self.namespace.as_ref() {
          self.invoke_schematic(invocation).await?
        } else {
          trace!(?invocation);
          self
            .providers
            .get(ns)
            .ok_or_else(|| Error::TargetNotFound(invocation.target.clone(), known_targets()))?
            .provider
            .handle(invocation, None)
            .instrument(trace_span!("provider invocation"))
            .await
            .map_err(|e| ExecutionError::ProviderError(e.to_string()))?
        }
      }
      _ => return Err(Error::TargetNotFound(invocation.target, known_targets())),
    };

    Ok(stream)
  }

  pub fn get_export_signature(&self) -> &ProviderSignature {
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
    for (ns, provider) in self.providers.providers() {
      debug!(namespace = %ns, "shutting down provider");
      if let Err(error) = provider
        .provider
        .shutdown()
        .await
        .map_err(|e| Error::ProviderShutdown(e.to_string()))
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
