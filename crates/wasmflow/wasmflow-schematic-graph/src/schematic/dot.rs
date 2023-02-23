use crate::{Node, NodeKind, PortDirection, Schematic};

pub(crate) fn render<DATA>(schematic: &Schematic<DATA>) -> String
where
  DATA: Clone,
{
  let mut lines = vec![format!("digraph \"{}\" {{", schematic.name())];
  for node in schematic.nodes() {
    lines.push(format!("subgraph \"cluster_{}\" {{", node.id()));
    lines.push(format!("label=\"{}\"", node.id()));
    let shape = match node.kind() {
      NodeKind::Input(_) => "house",
      NodeKind::Inherent(_) => "diamond",
      NodeKind::Output(_) => "invhouse",
      NodeKind::External(_) => "rectangle",
    };
    lines.push(format!("\"{}\"[shape=\"{}\"]", node.id(), shape));
    lines.append(&mut render_ports(node, PortDirection::In));
    lines.append(&mut render_ports(node, PortDirection::Out));
    lines.push("}".to_owned());
  }
  lines.append(&mut render_connections(schematic));

  render_connections(schematic);
  lines.push("}".to_owned());
  lines.join("\n")
}

fn render_connections<DATA>(schematic: &Schematic<DATA>) -> Vec<String>
where
  DATA: Clone,
{
  let mut lines = vec![];
  for conn in schematic.connections() {
    let from_node = &schematic.nodes()[conn.from().node_index()];
    let from_port = &from_node.outputs()[conn.from().port_index()];
    let to_node = &schematic.nodes()[conn.to().node_index()];
    let to_port = &to_node.inputs()[conn.to().port_index()];
    lines.push(format!(
      "\"{}.OUT.{}\" -> \"{}.IN.{}\"",
      from_node.id(),
      from_port.name(),
      to_node.id(),
      to_port.name(),
    ));
  }
  lines
}

fn render_ports<DATA>(node: &Node<DATA>, dir: PortDirection) -> Vec<String>
where
  DATA: Clone,
{
  let (label, shape, ports) = match dir {
    PortDirection::In => ("IN", "triangle", node.inputs()),
    PortDirection::Out => ("OUT", "invtriangle", node.outputs()),
  };
  let mut lines = vec![];
  for port in ports {
    match dir {
      PortDirection::In => lines.push(format!("\"{}.IN.{}\" -> \"{}\"", node.id(), port.name(), node.id(),)),
      PortDirection::Out => lines.push(format!("\"{}\" -> \"{}.OUT.{}\"", node.id(), node.id(), port.name(),)),
    };
  }
  lines.push(format!("subgraph \"{}.{}\" {{", node.id(), label));
  for port in ports {
    lines.push(format!(
      "\"{}.{}.{}\"[label=\"{}\", shape=\"{}\"]",
      node.id(),
      label,
      port.name(),
      port.name(),
      shape
    ));
  }
  lines.push("}\n".to_owned());
  lines
}
