pub mod types {
  #![allow(unused)]
  use super::AssociatedData;

  pub(crate) type Network = flow_graph::Network<AssociatedData>;
  pub(crate) type Operation = flow_graph::Node<AssociatedData>;
  pub(crate) type OperationPort = flow_graph::NodePort;
  pub(crate) type Schematic = flow_graph::Schematic<AssociatedData>;
  pub(crate) type Node = flow_graph::Node<AssociatedData>;
  pub(crate) type Port<'a> = flow_graph::iterators::Port<'a, AssociatedData>;
}
use std::collections::HashMap;
pub(crate) type AssociatedData = OperationSettings;

#[derive(Debug, Clone, Default)]
pub struct OperationSettings {
  pub(crate) config: LiquidOperationConfig,
  pub(crate) settings: Option<ExecutionSettings>,
}

impl OperationSettings {
  /// Initialize a new OperationSettings with the specified config and settings.
  pub(crate) fn new(config: LiquidOperationConfig, settings: Option<ExecutionSettings>) -> Self {
    Self { config, settings }
  }
}

#[derive(Debug, Clone, Default)]
pub struct LiquidOperationConfig {
  root: Option<RuntimeConfig>,
  template: Option<LiquidJsonConfig>,
  value: Option<RuntimeConfig>,
}

impl LiquidOperationConfig {
  #[must_use]
  pub fn new_template(template: Option<LiquidJsonConfig>) -> Self {
    Self {
      template,
      value: None,
      root: None,
    }
  }

  #[must_use]
  pub fn new_value(value: Option<RuntimeConfig>) -> Self {
    Self {
      template: None,
      value,
      root: None,
    }
  }

  pub fn render(&self, inherent: &InherentData) -> Result<Option<RuntimeConfig>, InterpreterError> {
    if let Some(template) = self.template() {
      Ok(Some(
        template
          .render(self.root.as_ref(), self.value.as_ref(), None, Some(inherent))
          .map_err(|e| InterpreterError::Configuration(e.to_string()))?,
      ))
    } else {
      Ok(self.value.clone())
    }
  }

  #[must_use]
  pub fn value(&self) -> Option<&RuntimeConfig> {
    self.value.as_ref()
  }

  #[must_use]
  pub fn template(&self) -> Option<&LiquidJsonConfig> {
    self.template.as_ref()
  }

  #[must_use]
  pub fn root(&self) -> Option<&RuntimeConfig> {
    self.root.as_ref()
  }

  pub fn set_root(&mut self, root: Option<RuntimeConfig>) {
    self.root = root;
  }

  pub fn set_template(&mut self, template: Option<LiquidJsonConfig>) {
    self.template = template;
  }

  pub fn set_value(&mut self, value: Option<RuntimeConfig>) {
    self.value = value;
  }
}

impl From<Option<LiquidJsonConfig>> for LiquidOperationConfig {
  fn from(value: Option<LiquidJsonConfig>) -> Self {
    LiquidOperationConfig {
      template: value,
      value: None,
      root: None,
    }
  }
}

impl From<Option<RuntimeConfig>> for LiquidOperationConfig {
  fn from(value: Option<RuntimeConfig>) -> Self {
    LiquidOperationConfig {
      template: None,
      value,
      root: None,
    }
  }
}

use flow_expression_parser::ast::{
  BlockExpression,
  ConnectionExpression,
  ConnectionTargetExpression,
  FlowExpression,
  InstancePort,
  InstanceTarget,
};
use flow_graph::NodeReference;
use serde_json::Value;
use types::*;
use wick_config::config::components::{ComponentConfig, OperationConfig};
use wick_config::config::{ComponentImplementation, ExecutionSettings, FlowOperation, LiquidJsonConfig};
use wick_packet::{InherentData, RuntimeConfig};

use crate::constants::{NS_CORE, NS_NULL};
use crate::error::InterpreterError;

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
  op_config: &LiquidOperationConfig,
) -> Result<(), flow_graph::error::Error> {
  scope.push(flow.name().to_owned());

  for flow in flow.flows_mut() {
    let scope = scope.clone();
    register_operation(scope, network, flow, op_config)?;
  }
  let name = scope.join("::");
  let mut schematic = Schematic::new(name, Default::default(), Default::default());

  for (name, def) in flow.instances().iter() {
    debug!(%name, config=?def.data(),settings=?def.settings(), "registering operation");
    let mut op_config = op_config.clone();
    op_config.set_template(def.data().cloned());

    schematic.add_external(
      name,
      NodeReference::new(def.component_id(), def.name()),
      OperationSettings::new(op_config, def.settings().cloned()),
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
    schematic.connect(from_port, to_port, Default::default())?;
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
        let id = format!("{}_pluck_{}_[{}]", schematic.name(), name, parts.join(","));
        let config = HashMap::from([(
          "path".to_owned(),
          Value::Array(parts.into_iter().map(Value::String).collect()),
        )]);
        schematic.add_external(
          &id,
          NodeReference::new("core", "pluck"),
          OperationSettings::new(Some(RuntimeConfig::from(config)).into(), None),
        );
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
      schematic.add_external(id, NodeReference::new(component_id, op), Default::default());
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
        schematic.add_external(id, NodeReference::new(component_id, op), Default::default());
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
      schematic.add_external(id_str, NodeReference::new(NS_NULL, "drop"), Default::default());
    }
    _ => {}
  }
  Ok(())
}

pub fn from_def(
  manifest: &mut wick_config::config::ComponentConfiguration,
) -> Result<Network, flow_graph::error::Error> {
  let mut network = Network::new(
    manifest.name().cloned().unwrap_or_default(),
    OperationSettings::new(manifest.root_config().cloned().into(), None),
  );

  let mut op_config = LiquidOperationConfig::default();
  op_config.set_root(manifest.root_config().cloned());

  if let ComponentImplementation::Composite(composite) = manifest.component_mut() {
    for flow in composite.operations_mut() {
      register_operation(vec![], &mut network, flow, &op_config)?;
    }
  }

  Ok(network)
}
