use std::collections::HashMap;
use std::sync::Arc;

use vino_schematic_graph::{Network, Schematic};
use vino_types::ComponentMap;

pub(crate) mod validator;
use super::error::Error;

#[must_use]
#[derive(Debug, Clone)]
pub(crate) struct Program {
  state: Arc<ProgramState>,
}

impl Program {
  pub(crate) fn new(network: Network, components: HashMap<String, ComponentMap>) -> Result<Self, Error> {
    let program = Self {
      state: ProgramState::new(network, components),
    };
    Ok(program)
  }

  pub(crate) fn state(&self) -> Arc<ProgramState> {
    self.state.clone()
  }

  pub(crate) fn schematics(&self) -> &[Schematic] {
    self.state.network.schematics()
  }

  pub(crate) fn validate(&self) -> Result<(), Error> {
    self::validator::validate(self)?;
    Ok(())
  }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct ProgramState {
  network: Network,
  components: HashMap<String, ComponentMap>,
}

impl ProgramState {
  pub(crate) fn new(network: Network, components: HashMap<String, ComponentMap>) -> Arc<Self> {
    Arc::new(Self { network, components })
  }
}
