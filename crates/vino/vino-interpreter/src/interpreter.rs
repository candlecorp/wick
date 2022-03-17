use std::collections::HashMap;

pub(crate) mod channel;
pub(crate) mod error;
pub(crate) mod event_loop;
pub(crate) mod executor;
pub(crate) mod program;
pub(crate) mod provider;

use vino_schematic_graph::Network;
use vino_types::ComponentMap;

use self::error::Error;
use self::event_loop::EventLoop;
use self::executor::SchematicExecutor;
use self::program::Program;
use self::provider::Providers;
use crate::InterpreterDispatchChannel;

#[must_use]
#[derive(Debug)]
pub struct Interpreter {
  #[allow(unused)]
  program: Program,
  subroutines: Vec<SchematicExecutor>,
  #[allow(unused)]
  channel: InterpreterDispatchChannel,
  event_loop: EventLoop,
}

impl Interpreter {
  pub fn new(network: Network, providers: Option<Providers>) -> Result<Self, Error> {
    trace_span!("interpreter", "start");
    let components: HashMap<String, ComponentMap> = providers
      .as_ref()
      .map(|providers| providers.component_hashmap())
      .unwrap_or_default();
    let program = Program::new(network, components)?;
    program.validate()?;
    let providers = providers.unwrap_or_default();

    let event_loop = EventLoop::new(providers);
    let channel = event_loop.channel();

    let subroutines = program
      .schematics()
      .iter()
      .map(|s| SchematicExecutor::new(s.clone(), channel.clone()))
      .collect();

    let interpreter = Self {
      subroutines,
      program,
      channel,
      event_loop,
    };

    Ok(interpreter)
  }

  #[must_use]
  #[instrument(skip(self))]
  pub fn schematic(&self, name: &str) -> Option<&SchematicExecutor> {
    self.subroutines.iter().find(|s| s.name() == name)
  }

  #[instrument(skip(self))]
  pub async fn start(&mut self) {
    self.event_loop.start().await;
  }

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
