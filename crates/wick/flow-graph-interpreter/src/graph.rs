pub mod types {
  use wick_packet::GenericConfig;

  pub(crate) static INHERENT_COMPONENT: usize = 2;
  pub(crate) type Network = flow_graph::Network<GenericConfig>;
  pub(crate) type Operation = flow_graph::Node<GenericConfig>;
  pub(crate) type OperationPort = flow_graph::NodePort;
  pub(crate) type Schematic = flow_graph::Schematic<GenericConfig>;
  pub(crate) type Port<'a> = flow_graph::iterators::Port<'a, GenericConfig>;
}

use std::collections::HashMap;

use flow_expression_parser::ast::{
  BlockExpression,
  ConnectionExpression,
  ConnectionTargetExpression,
  FlowExpression,
  InstancePort,
  InstanceTarget,
};
use flow_expression_parser::parse::CORE_ID;
use flow_graph::NodeReference;
use serde_json::Value;
use types::*;
use wick_config::config::{ComponentImplementation, FlowOperation};
use wick_packet::GenericConfig;

use crate::constants::{INTERNAL_ID_INHERENT, NS_CORE, NS_INTERNAL, NS_NULL};

#[derive(Debug)]
#[must_use]
pub(crate) struct Reference(NodeReference);

impl From<&NodeReference> for Reference {
  fn from(v: &NodeReference) -> Self {
    Self(v.clone())
  }
}

impl Reference {
  pub(crate) fn name(&self) -> &str {
    self.0.name()
  }
  pub(crate) fn namespace(&self) -> &str {
    self.0.component_id()
  }

  pub(crate) fn is_core_operation(&self, name: &str) -> bool {
    self.0.component_id() == NS_CORE && self.0.name() == name
  }
}

fn register_operation(
  mut scope: Vec<String>,
  network: &mut Network,
  flow: &mut FlowOperation,
) -> Result<(), flow_graph::error::Error> {
  scope.push(flow.name().to_owned());
  for flow in flow.flows_mut() {
    let scope = scope.clone();
    register_operation(scope, network, flow)?;
  }
  let name = scope.join("::");
  debug!(%name, "registering operation");
  let mut schematic = Schematic::new(name);

  let index = schematic.add_inherent(CORE_ID, NodeReference::new(NS_INTERNAL, INTERNAL_ID_INHERENT), None);

  trace!(index, name = INTERNAL_ID_INHERENT, "added inherent component");

  for (name, def) in flow.instances().iter() {
    schematic.add_external(
      name,
      NodeReference::new(def.component_id(), def.name()),
      def.data().cloned(),
    );
  }

  expand_expressions(&mut schematic, flow.expressions_mut())?;

  for expression in flow.expressions() {
    process_flow_expression(&mut schematic, expression)?;
  }
  network.add_schematic(schematic);
  Ok(())
}

fn process_flow_expression(schematic: &mut Schematic, expr: &FlowExpression) -> Result<(), flow_graph::error::Error> {
  match expr {
    FlowExpression::ConnectionExpression(expr) => process_connection_expression(schematic, expr)?,
    FlowExpression::BlockExpression(expr) => {
      for expr in expr.iter() {
        process_flow_expression(schematic, expr)?;
      }
    }
  }
  Ok(())
}

fn process_connection_expression(
  schematic: &mut Schematic,
  expr: &ConnectionExpression,
) -> Result<(), flow_graph::error::Error> {
  let from = expr.from();
  let to = expr.to();
  let to_port = schematic
    .find_mut(to.instance().id().unwrap())
    .map(|component| component.add_input(to.port().name()));

  if to_port.is_none() {
    error!("Missing downstream: instance {:?}", to);
    return Err(flow_graph::error::Error::MissingDownstream(
      to.instance().id().unwrap().to_owned(),
    ));
  }
  let to_port = to_port.unwrap();

  if let Some(component) = schematic.find_mut(from.instance().id().unwrap()) {
    let from_port = component.add_output(from.port().name());
    schematic.connect(from_port, to_port, None)?;
  } else {
    panic!("Can't find component {}", from.instance());
  }
  Ok(())
}

#[allow(clippy::option_if_let_else)]
fn expand_expressions(
  schematic: &mut Schematic,
  expressions: &mut [FlowExpression],
) -> Result<(), flow_graph::error::Error> {
  expand_port_paths(schematic, expressions)?;
  expand_inline_operations(schematic, expressions)?;

  Ok(())
}

#[allow(clippy::option_if_let_else)]
fn expand_port_paths(
  schematic: &mut Schematic,
  expressions: &mut [FlowExpression],
) -> Result<(), flow_graph::error::Error> {
  for expression in expressions.iter_mut() {
    if let FlowExpression::ConnectionExpression(expr) = expression {
      let (from, to) = expr.clone().into_parts();
      let (from_inst, from_port, _) = from.into_parts();
      let (to_inst, to_port, _) = to.into_parts();
      if let InstancePort::Path(name, parts) = from_port {
        let id = format!("{}_pluck_{}_{}", schematic.name(), name, parts.join(","));
        let config = GenericConfig::from(HashMap::from([("field".to_owned(), Value::String(parts.join(".")))]));
        schematic.add_external(&id, NodeReference::new("core", "pluck"), Some(config));
        *expression = FlowExpression::block(BlockExpression::new(vec![
          FlowExpression::connection(ConnectionExpression::new(
            ConnectionTargetExpression::new(from_inst, InstancePort::named(&name)),
            ConnectionTargetExpression::new(InstanceTarget::named(&id), InstancePort::named("input")),
          )),
          FlowExpression::connection(ConnectionExpression::new(
            ConnectionTargetExpression::new(InstanceTarget::named(&id), InstancePort::named("output")),
            ConnectionTargetExpression::new(to_inst, to_port),
          )),
        ]));
      }
    }
  }
  Ok(())
}

#[allow(clippy::option_if_let_else)]
fn expand_inline_operations(
  schematic: &mut Schematic,
  expressions: &mut [FlowExpression],
) -> Result<(), flow_graph::error::Error> {
  let mut inline_id = 0;
  let mut anonymous_path_ids: HashMap<String, (Vec<String>, String)> = HashMap::new();
  for expression in expressions.iter_mut() {
    expand_flow_expression(schematic, expression, &mut inline_id, &mut anonymous_path_ids)?;
  }
  Ok(())
}

fn expand_flow_expression(
  schematic: &mut Schematic,
  expression: &mut FlowExpression,
  inline_id: &mut usize,
  anonymous_path_ids: &mut HashMap<String, (Vec<String>, String)>,
) -> Result<(), flow_graph::error::Error> {
  match expression {
    FlowExpression::ConnectionExpression(expr) => {
      expand_operation(schematic, expr, inline_id, anonymous_path_ids)?;
    }
    FlowExpression::BlockExpression(block) => {
      for expr in block.iter_mut() {
        expand_flow_expression(schematic, expr, inline_id, anonymous_path_ids)?;
      }
    }
  }
  Ok(())
}

fn expand_operation(
  schematic: &mut Schematic,
  expr: &mut Box<ConnectionExpression>,
  inline_id: &mut usize,
  anonymous_path_ids: &mut HashMap<String, (Vec<String>, String)>,
) -> Result<(), flow_graph::error::Error> {
  if let InstanceTarget::Path(path, id) = expr.from().instance() {
    let id = id.as_ref().or(anonymous_path_ids.get(path).map(|(_, id)| id));
    #[allow(clippy::option_if_let_else)]
    if let Some(id) = id {
      let (component_id, op) = path.split_once("::").unwrap(); // unwrap OK if we come from a parsed config.
      schematic.add_external(id, NodeReference::new(component_id, op), None);
    } else {
      todo!()
    }
  }
  let to_port = expr.to().port().clone();
  let instance = expr.to_mut().instance_mut();
  match instance {
    InstanceTarget::Path(path, id) => {
      if let Some(id) = id {
        let (component_id, op) = path.split_once("::").unwrap(); // unwrap OK if we come from a parsed config.
        schematic.add_external(id, NodeReference::new(component_id, op), None);
      } else if let Some((ports, generated_id)) = anonymous_path_ids.get_mut(path) {
        if ports.contains(to_port.name()) {
          return Err(flow_graph::error::Error::AmbiguousOperation(path.clone()));
        }
        ports.push(to_port.name().clone());
        id.replace(generated_id.clone());
      }
    }
    InstanceTarget::Null(id) => {
      *inline_id += 1;
      let id_str = format!("drop_{}", inline_id);
      id.replace(id_str.clone());
      schematic.add_external(id_str, NodeReference::new(NS_NULL, "drop"), None);
    }
    _ => {}
  }
  Ok(())
}

pub fn from_def(
  manifest: &mut wick_config::config::ComponentConfiguration,
) -> Result<Network, flow_graph::error::Error> {
  let mut network = Network::new(manifest.name().cloned().unwrap_or_default());

  if let ComponentImplementation::Composite(composite) = manifest.component_mut() {
    for flow in composite.operations_mut().values_mut() {
      register_operation(vec![], &mut network, flow)?;
    }
  }

  Ok(network)
}
