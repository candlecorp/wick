use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use flow_expression_parser::ast::FlowExpression;
use flow_graph::iterators::SchematicHop;
use flow_graph::{Network, NodeReference, PortDirection, Schematic};
use wick_packet::OperationConfig;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counter {
  pub node_visits: usize,
  pub input_visits: usize,
  pub output_visits: usize,
  pub num_connections: usize,
  pub port_visits: usize,
  pub inputs: HashSet<String>,
  pub outputs: HashSet<String>,
  pub nodes: HashSet<String>,
}

#[allow(unused)]
pub fn hash_set(list: &[&str]) -> HashSet<String> {
  list.iter().map(|s| (*s).to_owned()).collect()
}

impl Counter {
  #[allow(unused)]
  pub fn walk_down(schematic: &Schematic<OperationConfig>) -> Self {
    let mut counter = Counter::default();
    let walker = schematic.walker();
    for hop in walker {
      println!("{}", hop);
      counter.count(&hop);
    }
    counter
  }
  #[allow(unused)]
  pub fn walk_up(schematic: &Schematic<OperationConfig>) -> Self {
    let mut counter = Counter::default();
    let walker = schematic.walk_from_output();
    for hop in walker {
      println!("{}", hop);
      counter.count(&hop);
    }
    counter
  }
  #[allow(unused)]
  pub fn count(&mut self, hop: &SchematicHop<OperationConfig>) {
    match hop {
      SchematicHop::Node(v) => {
        self.node_visits += 1;
        self.nodes.insert(v.name().to_owned());
      }
      SchematicHop::Port(v) => {
        match v.direction() {
          PortDirection::In => {
            self.input_visits += 1;
            self.inputs.insert(v.to_string());
          }
          PortDirection::Out => {
            self.output_visits += 1;
            self.outputs.insert(v.to_string());
          }
        }
        self.port_visits += 1;
      }
      SchematicHop::Ports(_) => (),
      SchematicHop::Connections(_) => (),
      SchematicHop::Connection(_) => self.num_connections += 1,
    };
  }
}

pub fn load<T: AsRef<Path>>(path: T) -> Result<wick_config::config::ComponentConfiguration> {
  Ok(wick_config::WickConfiguration::load_from_file_sync(path.as_ref())?.try_component_config()?)
}

pub fn from_manifest(network_def: &wick_config::config::ComponentConfiguration) -> Result<Network<OperationConfig>> {
  let mut network = Network::new(network_def.name().clone().unwrap_or_default());
  let network_def = network_def.try_composite()?;

  for flow in network_def.operations().values() {
    let mut schematic = Schematic::new(flow.name.clone());

    for (name, def) in flow.instances.iter() {
      schematic.add_external(name, NodeReference::new(&def.component_id, &def.name), def.data.clone());
    }

    for connection in &flow.expressions {
      if let FlowExpression::ConnectionExpression(connection) = connection {
        println!("{:?}", connection);
        let from = connection.from();
        let to = connection.to();
        let to_port = if let Some(node) = schematic.find_mut(to.instance().id()) {
          println!("{:?}", node);
          node.add_input(to.port())
        } else {
          panic!();
        };
        if let Some(node) = schematic.find_mut(from.instance().id()) {
          println!("{:?}", node);
          let from_port = node.add_output(from.port());
          schematic.connect(from_port, to_port, None)?;
        } else {
          // panic!();
        }
      }
    }
    network.add_schematic(schematic);
  }
  Ok(network)
}
