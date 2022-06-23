mod test;
use anyhow::Result;
use test::*;

#[test_logger::test(tokio::test)]
async fn test_walking() -> Result<()> {
  let manifest = load("./tests/manifests/v0/echo.wafl")?;
  let network = from_manifest(&manifest)?;
  let schematic = network.schematic("echo").unwrap();

  assert_eq!(schematic.name(), "echo");
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
  let manifest = load("./tests/manifests/v0/single-instance.wafl")?;
  let network = from_manifest(&manifest)?;
  let schematic = network.schematic("single-instance").unwrap();
  println!("components:{:#?}", schematic.components());

  assert_eq!(schematic.components().len(), 3);

  let counter = Counter::walk_down(schematic);

  let expected = Counter {
    component_visits: 3,
    input_visits: 2,
    output_visits: 3,
    num_connections: 2,
    port_visits: 5,
    inputs: hash_set(&["REF_ID_LOGGER.IN.input", "<output>.IN.output"]),
    outputs: hash_set(&["<input>.OUT.input", "REF_ID_LOGGER.OUT.output", "<output>.OUT.output"]),
    components: hash_set(&["<input>", "REF_ID_LOGGER", "<output>"]),
  };

  assert_eq!(counter, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_spread_io() -> Result<()> {
  let manifest = load("./tests/manifests/v0/spread-io.wafl")?;
  let network = from_manifest(&manifest)?;
  let schematic = network.schematic("spread-io").unwrap();

  let counter = Counter::walk_down(schematic);

  let expected = Counter {
    component_visits: 7,
    input_visits: 6,
    output_visits: 7,
    num_connections: 6,
    port_visits: 13,
    inputs: hash_set(&[
      "ZIP.IN.left",
      "<output>.IN.output",
      "COMP2.IN.input",
      "COMP1.IN.input",
      "ZIP.IN.right",
    ]),
    outputs: hash_set(&[
      "COMP1.OUT.output",
      "ZIP.OUT.output",
      "<output>.OUT.output",
      "<input>.OUT.input",
      "COMP2.OUT.output",
    ]),
    components: hash_set(&["<input>", "COMP1", "ZIP", "<output>", "COMP2"]),
  };

  assert_eq!(counter, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  let manifest = load("./tests/manifests/v0/senders.wafl")?;
  let network = from_manifest(&manifest)?;
  let schematic = network.schematic("test").unwrap();

  let counter = Counter::walk_up(schematic);

  let expected = Counter {
    component_visits: 2,
    input_visits: 1,
    output_visits: 1,
    num_connections: 1,
    port_visits: 2,
    inputs: hash_set(&["<output>.IN.output"]),
    outputs: hash_set(&["SENDER.OUT.output"]),
    components: hash_set(&["SENDER", "<output>"]),
  };

  assert_eq!(counter, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_dot() -> Result<()> {
  let manifest = load("./tests/manifests/v0/spread-io.wafl")?;
  let network = from_manifest(&manifest)?;
  let schematic = network.schematic("spread-io").unwrap();

  std::fs::write("./sample.dot", schematic.render_dot())?;
  Ok(())
}
