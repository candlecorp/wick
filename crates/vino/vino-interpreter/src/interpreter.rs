pub(crate) mod channel;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;
pub(crate) mod provider;

use std::sync::Arc;

use vino_entity::Entity;
use vino_transport::{Invocation, TransportStream};
use vino_types::ProviderSignature;

use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use self::provider::schematic_provider::SELF_NAMESPACE;
use self::provider::Providers;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterChannel;
use crate::interpreter::provider::provider_provider::{ProviderProvider, PROVIDERPROVIDER_NAMESPACE};
use crate::interpreter::provider::schematic_provider::SchematicProvider;
use crate::{ExecutionError, InterpreterDispatchChannel, Provider, ProviderNamespace};

#[must_use]
#[derive()]
pub struct Interpreter {
  program: Program,
  event_loop: EventLoop,
  signature: ProviderSignature,
  providers: Arc<Providers>,
  self_provider: Arc<SchematicProvider>,
  dispatcher: InterpreterDispatchChannel,
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
  pub fn new(network: Network, providers: Option<Providers>) -> Result<Self, Error> {
    trace_span!("interpreter", "start");
    let mut providers = providers.unwrap_or_default();

    // Add the provider:: provider
    let provider_provider = ProviderProvider::new(&providers);
    providers.add(ProviderNamespace {
      namespace: PROVIDERPROVIDER_NAMESPACE.to_owned(),
      provider: Arc::new(Box::new(provider_provider)),
    });

    let signatures = providers.provider_signatures();

    let program = Program::new(network, signatures)?;

    program.validate()?;

    let channel = InterpreterChannel::new();
    let dispatcher = channel.dispatcher();

    // Make the self:: provider
    let providers = Arc::new(providers);
    let self_provider = SchematicProvider::new(providers.clone(), program.state(), &dispatcher);
    let self_signature = self_provider.list().clone();

    debug!(?self_signature, "signature");

    let event_loop = EventLoop::new(channel);

    let interpreter = Self {
      program,
      dispatcher,
      signature: self_signature,
      providers,
      self_provider,
      event_loop,
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
      .ok_or_else(|| Error::TargetNotFound(invocation.target.clone()))?;
    let executor = SchematicExecutor::new(schematic.clone(), dispatcher.clone());

    Ok(
      executor
        .invoke(invocation, self.providers.clone(), self.self_provider.clone())
        .await?,
    )
  }

  #[instrument(skip(self))]
  pub async fn invoke(&self, invocation: Invocation) -> Result<TransportStream, Error> {
    trace!(?invocation);
    let stream = match &invocation.target {
      Entity::Schematic(_) => self.invoke_schematic(invocation).await?,
      Entity::Component(ns, _) => {
        if ns == SELF_NAMESPACE || ns == Entity::LOCAL {
          self.invoke_schematic(invocation).await?
        } else {
          self
            .providers
            .get(ns)
            .ok_or_else(|| Error::TargetNotFound(invocation.target.clone()))?
            .provider
            .handle(invocation, None)
            .await
            .map_err(|e| ExecutionError::ProviderError(e.to_string()))?
        }
      }
      _ => return Err(Error::TargetNotFound(invocation.target)),
    };

    Ok(stream)
  }

  pub fn get_export_signature(&self) -> &ProviderSignature {
    &self.signature
  }

  #[instrument(skip(self))]
  pub async fn start(&mut self) {
    self.event_loop.start().await;
  }

  #[instrument(skip(self))]
  pub async fn shutdown(self) -> Result<(), Error> {
    self.event_loop.shutdown().await?;

    Ok(())
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
