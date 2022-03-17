use crate::{Component, ComponentKind, PortDirection, Schematic};

pub(crate) fn render(schematic: &Schematic) -> String {
  let mut lines = vec![format!("digraph \"{}\" {{", schematic.name())];
  for component in schematic.components() {
    lines.push(format!("subgraph \"cluster_{}\" {{", component.name()));
    lines.push(format!("label=\"{}\"", component.name()));
    let shape = match component.kind() {
      ComponentKind::Input => "house",
      ComponentKind::Output => "invhouse",
      ComponentKind::External(_) => "rectangle",
    };
    lines.push(format!("\"{}\"[shape=\"{}\"]", component.name(), shape));
    lines.append(&mut render_ports(component, PortDirection::In));
    lines.append(&mut render_ports(component, PortDirection::Out));
    lines.push("}".to_owned());
  }
  lines.append(&mut render_connections(schematic));

  render_connections(schematic);
  lines.push("}".to_owned());
  lines.join("\n")
}

fn render_connections(schematic: &Schematic) -> Vec<String> {
  let mut lines = vec![];
  for conn in schematic.connections() {
    let from_component = &schematic.components()[conn.from().component_index()];
    let from_port = &from_component.outputs()[conn.from().port_index()];
    let to_component = &schematic.components()[conn.to().component_index()];
    let to_port = &to_component.inputs()[conn.to().port_index()];
    lines.push(format!(
      "\"{}.OUT.{}\" -> \"{}.IN.{}\"",
      from_component.name(),
      from_port.name(),
      to_component.name(),
      to_port.name(),
    ));
  }
  lines
}

fn render_ports(component: &Component, dir: PortDirection) -> Vec<String> {
  let (label, shape, ports) = match dir {
    PortDirection::In => ("IN", "triangle", component.inputs()),
    PortDirection::Out => ("OUT", "invtriangle", component.outputs()),
  };
  let mut lines = vec![];
  for port in ports {
    match dir {
      PortDirection::In => lines.push(format!(
        "\"{}.IN.{}\" -> \"{}\"",
        component.name(),
        port.name(),
        component.name(),
      )),
      PortDirection::Out => lines.push(format!(
        "\"{}\" -> \"{}.OUT.{}\"",
        component.name(),
        component.name(),
        port.name(),
      )),
    };
  }
  lines.push(format!("subgraph \"{}.{}\" {{", component.name(), label));
  for port in ports {
    lines.push(format!(
      "\"{}.{}.{}\"[label=\"{}\", shape=\"{}\"]",
      component.name(),
      label,
      port.name(),
      port.name(),
      shape
    ));
  }
  lines.push("}\n".to_owned());
  lines
}
