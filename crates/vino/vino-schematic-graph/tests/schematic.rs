use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
// use pretty_assertions::assert_eq;
use vino_manifest::Loadable;
use vino_schematic_graph::iterators::SchematicHop;
use vino_schematic_graph::{ExternalReference, Network, PortDirection, Schematic};

#[derive(Debug, Default, PartialEq, Eq)]
struct Counter {
  component_visits: usize,
  input_visits: usize,
  output_visits: usize,
  num_connections: usize,
  port_visits: usize,
  inputs: HashSet<String>,
  outputs: HashSet<String>,
  components: HashSet<String>,
}

fn make_hash(list: &[&str]) -> HashSet<String> {
  list.iter().map(|s| (*s).to_owned()).collect()
}

impl Counter {
  fn walk(schematic: &Schematic) -> Self {
    let mut counter = Counter::default();
    let walker = schematic.walker();
    for hop in walker {
      println!("{}", hop);
      counter.count(&hop);
    }
    counter
  }
  fn count(&mut self, hop: &SchematicHop) {
    match hop {
      SchematicHop::SchematicInput(v) => {
        // self.component_visits += 1;
        self.components.insert(v.name().to_owned());
      }
      SchematicHop::SchematicOutput(v) => {
        // self.component_visits += 1;
        self.components.insert(v.name().to_owned());
      }
      SchematicHop::Component(v) => {
        self.component_visits += 1;
        self.components.insert(v.name().to_owned());
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

fn load<T: AsRef<Path>>(path: T) -> Result<vino_manifest::HostManifest> {
  Ok(vino_manifest::HostManifest::load_from_file(path.as_ref())?)
}

fn from_manifest(network_manifest: &vino_manifest::NetworkDefinition) -> Result<Network> {
  let mut network = Network::new(network_manifest.name.clone().unwrap_or_default());

  for m in &network_manifest.schematics {
    let mut schematic = Schematic::new(m.name.clone());

    for (name, def) in m.instances.iter() {
      schematic.add_or_get_instance(name, ExternalReference::new(&def.namespace, &def.name));
    }

    for connection in &m.connections {
      println!("{}", connection);
      let from = &connection.from;
      let to = &connection.to;
      let from_port = if let Some(component) = schematic.find_mut(from.get_instance()) {
        println!("{:?}", component);
        component.add_output(from.get_port())
      } else {
        panic!();
      };
      let to_port = if let Some(component) = schematic.find_mut(to.get_instance()) {
        println!("{:?}", component);
        component.add_input(to.get_port())
      } else {
        panic!();
      };

      schematic.connect(from_port, to_port)?;
    }
    network.add_schematic(schematic);
  }
  Ok(network)
}

#[test_logger::test(tokio::test)]
async fn test_walking() -> Result<()> {
  let manifest = load("./tests/manifests/v0/echo.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let schematic = network.schematic("echo").unwrap();

  assert_eq!(schematic.name(), "echo");
  assert_eq!(schematic.components().len(), 2);
  let input_node = schematic.input();

  let schematic_output = schematic.output();

  let port = input_node.find_output("input").unwrap();

  let mut downstreams = schematic.downstream_connections(port).unwrap();
  let downstream_connection = downstreams.next().unwrap();

  let downstream_port = downstream_connection.to();
  assert_eq!(downstream_port.name(), "output");

  let downstream_component = downstream_port.component();
  assert_eq!(downstream_component.name(), "<output>");

  assert_eq!(schematic_output, downstream_component.inner());

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_iterator() -> Result<()> {
  let manifest = load("./tests/manifests/v0/single-instance.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let schematic = network.schematic("single-instance").unwrap();

  assert_eq!(schematic.components().len(), 3);

  let counter = Counter::walk(schematic);

  let expected = Counter {
    component_visits: 1,
    input_visits: 3,
    output_visits: 3,
    num_connections: 2,
    port_visits: 6,
    inputs: make_hash(&["<input>.IN.input", "REF_ID_LOGGER.IN.input", "<output>.IN.output"]),
    outputs: make_hash(&["<input>.OUT.input", "REF_ID_LOGGER.OUT.output", "<output>.OUT.output"]),
    components: make_hash(&["<input>", "REF_ID_LOGGER", "<output>"]),
  };

  assert_eq!(counter, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_spread_io() -> Result<()> {
  let manifest = load("./tests/manifests/v0/spread-io.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let schematic = network.schematic("spread-io").unwrap();

  let counter = Counter::walk(schematic);

  let expected = Counter {
    component_visits: 2,
    input_visits: 5,
    output_visits: 5,
    num_connections: 4,
    port_visits: 10,
    inputs: make_hash(&[
      "<input>.IN.input",
      "COMP1.IN.input",
      "COMP2.IN.input",
      "<output>.IN.output",
    ]),
    outputs: make_hash(&[
      "<input>.OUT.input",
      "COMP1.OUT.output",
      "COMP2.OUT.output",
      "<output>.OUT.output",
    ]),
    components: make_hash(&["<input>", "COMP1", "COMP2", "<output>"]),
  };

  assert_eq!(counter, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_dot() -> Result<()> {
  let manifest = load("./tests/manifests/v0/spread-io.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let schematic = network.schematic("spread-io").unwrap();

  std::fs::write("./sample.dot", schematic.render_dot())?;
  Ok(())
}
