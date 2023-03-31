use std::collections::HashSet;

use flow_graph::iterators::{SchematicHop, WalkDirection};
use flow_graph::{NodeKind, PortDirection};
use wick_interface_types::{ComponentSignature, Field, OperationSignature, TypeSignature};

use crate::constants::*;
use crate::error::ValidationError;
use crate::graph::types::*;

pub(crate) mod validator;
use super::collections::{get_id, ComponentMap};
use super::error::Error;

#[must_use]
#[derive(Debug)]
pub(crate) struct Program {
  state: ProgramState,
}

impl Program {
  pub(crate) fn new(network: Network, mut collections: ComponentMap) -> Result<Self, Error> {
    generate_self_signature(&network, &mut collections).map_err(Error::EarlyError)?;

    let program = Self {
      state: ProgramState::new(network, collections),
    };
    Ok(program)
  }

  pub(crate) fn state(&self) -> &ProgramState {
    &self.state
  }

  pub(crate) fn operations(&self) -> &[Schematic] {
    self.state.network.schematics()
  }

  pub(crate) fn validate(&self) -> Result<(), Error> {
    self::validator::validate(self)?;
    Ok(())
  }
}

fn get_resolution_order(network: &Network) -> Result<Vec<Vec<&Schematic>>, ValidationError> {
  let mut order = vec![];
  let mut will_resolve = HashSet::new();
  let mut schematics: Vec<&Schematic> = network.schematics().iter().collect();
  let mut cycle = 0;
  let mut num_unresolved = schematics.len();
  while cycle < 5 {
    let mut unresolved = vec![];
    let mut next_batch = vec![];
    for schematic in schematics {
      let mut resolvable = true;

      for component in schematic.nodes() {
        match component.kind() {
          NodeKind::External(ext) => {
            let references_self = ext.component_id() == NS_SELF;
            let reference_will_have_resolved = will_resolve.contains(ext.name());

            if references_self && !reference_will_have_resolved {
              resolvable = false;
            }
          }
          _ => continue,
        }
      }

      if resolvable {
        will_resolve.insert(schematic.name());
        next_batch.push(schematic);
      } else {
        unresolved.push(schematic);
      }
    }
    if !next_batch.is_empty() {
      order.push(next_batch);
    }
    schematics = unresolved;
    if schematics.is_empty() {
      break;
    }
    if num_unresolved == schematics.len() {
      cycle += 1;
    } else {
      num_unresolved = schematics.len();
    }
  }
  if cycle >= 5 {
    Err(ValidationError::NetworkUnresolvable(
      schematics.iter().map(|s| s.name().to_owned()).collect(),
    ))
  } else {
    Ok(order)
  }
}

fn generate_self_signature(network: &Network, collections: &mut ComponentMap) -> Result<(), ValidationError> {
  let map = ComponentSignature::new(NS_SELF);
  collections.insert(NS_SELF.to_owned(), map);
  let resolution_order = get_resolution_order(network)?;

  for batch in resolution_order {
    for schematic in batch {
      let signature = get_schematic_signature(schematic, collections)?;
      let map = collections.get_mut(NS_SELF).unwrap();
      map.operations.push(signature);
    }
  }
  Ok(())
}

fn get_schematic_signature(
  schematic: &Schematic,
  collections: &ComponentMap,
) -> Result<OperationSignature, ValidationError> {
  let mut schematic_signature = OperationSignature::new(schematic.name());

  for port in schematic.input().outputs() {
    for hop in schematic.walk_from_port(port, WalkDirection::Down).skip(1) {
      let signature = match hop {
        SchematicHop::Port(p) => {
          if p.direction() == PortDirection::In {
            let signature = get_signature(schematic.name(), &p, PortDirection::In, collections)?;
            match signature {
              Some(sig) => sig,
              None => continue,
            }
          } else {
            continue;
          }
        }
        _ => continue,
      };
      schematic_signature.inputs.push(Field::new(port.name(), signature));
    }
  }

  for port in schematic.output().inputs() {
    for hop in schematic.walk_from_port(port, WalkDirection::Up).skip(1) {
      let signature = match hop {
        SchematicHop::Port(p) => {
          if p.direction() == PortDirection::Out {
            let signature = get_signature(schematic.name(), &p, PortDirection::Out, collections)?;
            match signature {
              Some(sig) => sig,
              None => continue,
            }
          } else {
            continue;
          }
        }
        _ => continue,
      };
      schematic_signature.outputs.push(Field::new(port.name(), signature));
    }
  }
  Ok(schematic_signature)
}

fn get_signature(
  schematic: &str,
  p: &Port,
  kind: PortDirection,
  components: &ComponentMap,
) -> Result<Option<TypeSignature>, ValidationError> {
  let name = p.name();
  match p.node().kind() {
    NodeKind::Input(_) => match kind {
      PortDirection::In => Ok(None),
      PortDirection::Out => Ok(Some(TypeSignature::Value)),
    },

    NodeKind::Output(_) => match kind {
      PortDirection::Out => Ok(None),
      PortDirection::In => Ok(Some(TypeSignature::Value)),
    },
    NodeKind::External(ext) | NodeKind::Inherent(ext) => {
      let ext_collection = components
        .get(ext.component_id())
        .ok_or_else(|| ValidationError::ComponentIdNotFound(ext.component_id().to_owned()))?;

      let operation = p.node();

      let id = get_id(ext.component_id(), ext.name(), schematic, operation.name());

      let operation =
        ext_collection
          .operations
          .iter()
          .find(|op| op.name == id)
          .ok_or(ValidationError::MissingOperation {
            component: ext.component_id().to_owned(),
            name: id.clone(),
          })?;

      let sig = match kind {
        PortDirection::In => operation.inputs.iter().find(|p| p.name == name).map(|p| p.ty.clone()),
        PortDirection::Out => operation.outputs.iter().find(|p| p.name == name).map(|p| p.ty.clone()),
      };

      Ok(Some(sig.ok_or(ValidationError::MissingConnection {
        operation: ext.name().to_owned(),
        component: ext.component_id().to_owned(),
        port: name.to_owned(),
      })?))
    }
  }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct ProgramState {
  pub(crate) network: Network,
  pub(crate) components: ComponentMap,
}

impl ProgramState {
  pub(crate) fn new(network: Network, components: ComponentMap) -> Self {
    Self { network, components }
  }
}
