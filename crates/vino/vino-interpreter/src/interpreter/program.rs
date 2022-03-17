use std::collections::HashSet;

use vino_schematic_graph::iterators::{SchematicHop, WalkDirection};
use vino_schematic_graph::{ComponentKind, PortDirection};
use vino_types::{ComponentSignature, MapWrapper, ProviderMap, ProviderSignature, TypeSignature};

use crate::graph::types::*;
use crate::interpreter::provider::schematic_provider::SELF_NAMESPACE;

pub(crate) mod validator;
use super::error::Error;
use crate::ValidationError;

#[must_use]
#[derive(Debug)]
pub(crate) struct Program {
  state: ProgramState,
}

impl Program {
  pub(crate) fn new(network: Network, mut providers: ProviderMap) -> Result<Self, Error> {
    generate_self_signature(&network, &mut providers).map_err(Error::EarlyError)?;

    let program = Self {
      state: ProgramState::new(network, providers),
    };
    Ok(program)
  }

  pub(crate) fn state(&self) -> &ProgramState {
    &self.state
  }

  pub(crate) fn schematics(&self) -> &[Schematic] {
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

      for component in schematic.components() {
        match component.kind() {
          ComponentKind::External(ext) => {
            let references_self = ext.namespace() == SELF_NAMESPACE;
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

fn generate_self_signature(network: &Network, providers: &mut ProviderMap) -> Result<(), ValidationError> {
  let map = ProviderSignature::new(SELF_NAMESPACE);
  providers.insert(SELF_NAMESPACE, map);
  let resolution_order = get_resolution_order(network)?;

  for batch in resolution_order {
    for schematic in batch {
      let signature = get_schematic_signature(schematic, providers)?;
      let map = providers.get_inner_mut().get_mut(SELF_NAMESPACE).unwrap();
      map.components.insert(schematic.name(), signature);
    }
  }
  Ok(())
}

fn get_schematic_signature(
  schematic: &Schematic,
  providers: &ProviderMap,
) -> Result<ComponentSignature, ValidationError> {
  let mut schematic_signature = ComponentSignature::new(schematic.name());

  for port in schematic.input().outputs() {
    for hop in schematic.walk_from_port(port, WalkDirection::Down).skip(1) {
      let signature = match hop {
        SchematicHop::Port(p) => {
          if p.direction() == PortDirection::In {
            let signature = get_signature(&p, PortDirection::In, providers)?;
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
      schematic_signature.inputs.insert(port.name(), signature);
    }
  }

  for port in schematic.output().inputs() {
    for hop in schematic.walk_from_port(port, WalkDirection::Up).skip(1) {
      let signature = match hop {
        SchematicHop::Port(p) => {
          if p.direction() == PortDirection::Out {
            let signature = get_signature(&p, PortDirection::Out, providers)?;
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
      schematic_signature.outputs.insert(port.name(), signature);
    }
  }
  Ok(schematic_signature)
}

fn get_signature(
  p: &Port,
  kind: PortDirection,
  providers: &ProviderMap,
) -> Result<Option<TypeSignature>, ValidationError> {
  let name = p.name();
  match p.component().kind() {
    ComponentKind::Input => match kind {
      PortDirection::In => Ok(None),
      PortDirection::Out => Ok(Some(TypeSignature::Raw)),
    },

    ComponentKind::Output => match kind {
      PortDirection::Out => Ok(None),
      PortDirection::In => Ok(Some(TypeSignature::Raw)),
    },
    ComponentKind::External(ext) | ComponentKind::Inherent(ext) => {
      let ext_provider = providers
        .get(ext.namespace())
        .ok_or_else(|| ValidationError::MissingProvider(ext.namespace().to_owned()))?;
      let component = ext_provider
        .components
        .get(ext.name())
        .ok_or(ValidationError::MissingComponent {
          name: ext.name().to_owned(),
          namespace: ext.namespace().to_owned(),
        })?;

      let sig = match kind {
        PortDirection::In => component.inputs.get(name),
        PortDirection::Out => component.outputs.get(name),
      };

      Ok(Some(sig.cloned().ok_or(ValidationError::MissingPort {
        component: ext.name().to_owned(),
        namespace: ext.namespace().to_owned(),
        port: name.to_owned(),
      })?))
    }
  }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct ProgramState {
  pub(crate) network: Network,
  pub(crate) providers: ProviderMap,
}

impl ProgramState {
  pub(crate) fn new(network: Network, providers: ProviderMap) -> Self {
    Self { network, providers }
  }
}
