use crate::{Component, ComponentKind, PortDirection, Schematic};

pub(crate) fn render<DATA>(schematic: &Schematic<DATA>) -> String
where
  DATA: Clone,
{
  let mut lines = vec![format!("digraph \"{}\" {{", schematic.name())];
  for component in schematic.components() {
    lines.push(format!("subgraph \"cluster_{}\" {{", component.id()));
    lines.push(format!("label=\"{}\"", component.id()));
    let shape = match component.kind() {
      ComponentKind::Input(_) => "house",
      ComponentKind::Inherent(_) => "diamond",
      ComponentKind::Output(_) => "invhouse",
      ComponentKind::External(_) => "rectangle",
    };
    lines.push(format!("\"{}\"[shape=\"{}\"]", component.id(), shape));
    lines.append(&mut render_ports(component, PortDirection::In));
    lines.append(&mut render_ports(component, PortDirection::Out));
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
    let from_component = &schematic.components()[conn.from().component_index()];
    let from_port = &from_component.outputs()[conn.from().port_index()];
    let to_component = &schematic.components()[conn.to().component_index()];
    let to_port = &to_component.inputs()[conn.to().port_index()];
    lines.push(format!(
      "\"{}.OUT.{}\" -> \"{}.IN.{}\"",
      from_component.id(),
      from_port.name(),
      to_component.id(),
      to_port.name(),
    ));
  }
  lines
}

fn render_ports<DATA>(component: &Component<DATA>, dir: PortDirection) -> Vec<String>
where
  DATA: Clone,
{
  let (label, shape, ports) = match dir {
    PortDirection::In => ("IN", "triangle", component.inputs()),
    PortDirection::Out => ("OUT", "invtriangle", component.outputs()),
  };
  let mut lines = vec![];
  for port in ports {
    match dir {
      PortDirection::In => lines.push(format!(
        "\"{}.IN.{}\" -> \"{}\"",
        component.id(),
        port.name(),
        component.id(),
      )),
      PortDirection::Out => lines.push(format!(
        "\"{}\" -> \"{}.OUT.{}\"",
        component.id(),
        component.id(),
        port.name(),
      )),
    };
  }
  lines.push(format!("subgraph \"{}.{}\" {{", component.id(), label));
  for port in ports {
    lines.push(format!(
      "\"{}.{}.{}\"[label=\"{}\", shape=\"{}\"]",
      component.id(),
      label,
      port.name(),
      port.name(),
      shape
    ));
  }
  lines.push("}\n".to_owned());
  lines
}
