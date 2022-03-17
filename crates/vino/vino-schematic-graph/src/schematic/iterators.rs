mod component;
mod connection;
mod port;

pub use component::*;
pub use connection::*;
pub use port::*;
use tracing::instrument;

use crate::component::{ComponentKind, ComponentPort};
use crate::port::PortDirection;
use crate::{Connection, ConnectionIndex, PortReference, Schematic};

#[derive(Debug, Clone)]
pub enum SchematicHop<'graph> {
  SchematicInput(ComponentHop<'graph>),
  SchematicOutput(ComponentHop<'graph>),
  Component(ComponentHop<'graph>),
  Port(Port<'graph>),
  Ports(Ports<'graph>),
  Connections(Connections<'graph>),
  Connection(ConnectionRef<'graph>),
}

impl<'graph> std::fmt::Display for SchematicHop<'graph> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        SchematicHop::SchematicInput(v) => format!("SchematicInput:{}", v.name()),
        SchematicHop::SchematicOutput(v) => format!("SchematicOutput:{}", v.name()),
        SchematicHop::Component(v) => format!("Component:{}", v.name()),
        SchematicHop::Port(v) => format!("Port:{}", v),
        SchematicHop::Ports(v) => format!("Ports:{}", v),
        SchematicHop::Connections(v) => format!("Connections:{}", v),
        SchematicHop::Connection(v) => format!("Connection:{}", v),
      }
    )
  }
}

impl<'graph> From<ComponentHop<'graph>> for SchematicHop<'graph> {
  fn from(component: ComponentHop<'graph>) -> Self {
    match component.kind() {
      ComponentKind::Input => Self::SchematicInput(component),
      ComponentKind::Output => Self::SchematicOutput(component),
      ComponentKind::External(_) => Self::Component(component),
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum WalkDirection {
  Up,
  Down,
}

#[derive(Debug)]
#[must_use]
pub struct SchematicWalker<'graph> {
  schematic: &'graph Schematic,
  last_hop: Option<SchematicHop<'graph>>,
  hop_queue: Vec<SchematicHop<'graph>>,
  direction: WalkDirection,
}

impl<'graph> SchematicWalker<'graph> {
  pub fn new(schematic: &'graph Schematic) -> Self {
    let inputs = ComponentHop::new(schematic, schematic.input).into_inputs();
    let hop_queue = vec![SchematicHop::Ports(inputs)];
    Self {
      schematic,
      last_hop: None,
      hop_queue,
      direction: WalkDirection::Down,
    }
  }

  pub fn from_port(schematic: &'graph Schematic, port: PortReference, direction: WalkDirection) -> Self {
    let port = Port::new(schematic, port);
    let hop_queue = vec![SchematicHop::Port(port)];
    Self {
      schematic,
      last_hop: None,
      hop_queue,
      direction,
    }
  }
}

impl<'graph> Iterator for SchematicWalker<'graph> {
  type Item = SchematicHop<'graph>;

  fn next(&mut self) -> Option<SchematicHop<'graph>> {
    let last_hop = self.last_hop.take();
    let (mut next_hop, branch) = walk(self.schematic, last_hop, self.direction);

    if let Some(branch) = branch {
      self.hop_queue.push(branch);
    }
    if next_hop.is_none() {
      next_hop = self.hop_queue.pop();
    }
    self.last_hop = next_hop.clone();
    next_hop
  }
}

#[allow(clippy::too_many_lines)]
#[instrument(skip(schematic))]
fn walk<'graph>(
  schematic: &'graph Schematic,
  hop: Option<SchematicHop<'graph>>,
  direction: WalkDirection,
) -> (Option<SchematicHop<'graph>>, Option<SchematicHop<'graph>>) {
  match hop {
    Some(hop) => match hop {
      SchematicHop::SchematicOutput(v) | SchematicHop::SchematicInput(v) | SchematicHop::Component(v) => {
        let ports = match direction {
          WalkDirection::Up => v.into_inputs(),
          WalkDirection::Down => v.into_outputs(),
        };
        if ports.is_empty() {
          (None, None)
        } else {
          (Some(SchematicHop::Ports(ports)), None)
        }
      }
      SchematicHop::Port(v) => match direction {
        WalkDirection::Down => match v.port.direction {
          PortDirection::In => {
            let component = ComponentHop::new(schematic, v.port.component_index);
            (Some(component.into()), None)
          }
          PortDirection::Out => {
            let connections = schematic
              .get(v.port.component_index)
              .and_then(|component| component.output_connections(v.port.port_index))
              .map(|indices| Connections::new(schematic, indices.clone()))
              .unwrap();

            if connections.is_empty() {
              (None, None)
            } else {
              (Some(SchematicHop::Connections(connections)), None)
            }
          }
        },
        WalkDirection::Up => match v.port.direction {
          PortDirection::In => {
            let connections = schematic
              .get(v.port.component_index)
              .and_then(|component| component.input_connections(v.port.port_index))
              .map(|indices| Connections::new(schematic, indices.clone()))
              .unwrap();
            if connections.is_empty() {
              (None, None)
            } else {
              (Some(SchematicHop::Connections(connections)), None)
            }
          }
          PortDirection::Out => {
            let component = ComponentHop::new(schematic, v.port.component_index);
            (Some(component.into()), None)
          }
        },
      },
      SchematicHop::Ports(mut v) => {
        if v.direction.is_none() {
          return (None, None);
        }
        let port_direction = v.direction.unwrap();
        match direction {
          WalkDirection::Up => match port_direction {
            PortDirection::In => {
              let result = v.next();
              let rest = if result.is_none() {
                None
              } else {
                Some(SchematicHop::Ports(v))
              };
              (result.map(SchematicHop::Port), rest)
            }
            PortDirection::Out => {
              let component = ComponentHop::new(schematic, v.component_index);
              (Some(component.into()), None)
            }
          },
          WalkDirection::Down => match port_direction {
            PortDirection::In => {
              let component = ComponentHop::new(schematic, v.component_index);
              (Some(component.into()), None)
            }
            PortDirection::Out => {
              let result = v.next();
              let rest = if result.is_none() {
                None
              } else {
                Some(SchematicHop::Ports(v))
              };
              (result.map(SchematicHop::Port), rest)
            }
          },
        }
      }
      SchematicHop::Connection(v) => {
        let connection = schematic.connections[v.index];
        match direction {
          WalkDirection::Up => {
            let port = Port::new(schematic, connection.from);
            (Some(SchematicHop::Port(port)), None)
          }
          WalkDirection::Down => {
            let port = Port::new(schematic, connection.to);
            (Some(SchematicHop::Port(port)), None)
          }
        }
      }
      SchematicHop::Connections(mut v) => {
        let result = v.next();
        let rest = if result.is_none() {
          None
        } else {
          Some(SchematicHop::Connections(v))
        };

        (result.map(SchematicHop::Connection), rest)
      }
    },
    None => (None, None),
  }
}

fn get_ports_component<'graph, 'b>(schematic: &'graph Schematic, port: &'b PortReference) -> &'graph ComponentPort {
  match port.direction {
    PortDirection::In => &schematic.components[port.component_index].inputs()[port.port_index],
    PortDirection::Out => &schematic.components[port.component_index].outputs()[port.port_index],
  }
}

fn get_port_name<'graph, 'b>(schematic: &'graph Schematic, port: &'b PortReference) -> &'graph str {
  match port.direction {
    PortDirection::In => schematic.components[port.component_index].inputs()[port.port_index].name(),
    PortDirection::Out => schematic.components[port.component_index].outputs()[port.port_index].name(),
  }
}

fn get_port_connections<'graph, 'b>(schematic: &'graph Schematic, port: &'b PortReference) -> Connections<'graph> {
  let direction = port.direction;
  schematic
    .get(port.component_index)
    .and_then(|component| match direction {
      PortDirection::In => component.input_connections(port.port_index),
      PortDirection::Out => component.output_connections(port.port_index),
    })
    .map(|indices| Connections::new(schematic, indices.clone()))
    .unwrap()
}

fn get_connection(schematic: &Schematic, index: ConnectionIndex) -> &Connection {
  &schematic.connections[index]
}
