use flow_component::Operation;
use flow_expression_parser::ast::{FlowExpression, InstanceTarget, TargetId};
use flow_graph::NodeReference;
use wick_config::config::ExecutionSettings;

use super::types::{Node, Schematic};
use super::{GraphError, LiquidOperationConfig, NodeDecorator};
use crate::graph::OperationSettings;
use crate::interpreter::components;
use crate::interpreter::components::null::NullComponent;
use crate::HandlerMap;

pub(crate) trait ParseHelper {
  fn get_node<'a>(&'a self, instance: &InstanceTarget) -> Result<&'a Node, GraphError>;
}

impl ParseHelper for Schematic {
  fn get_node<'a>(&'a self, instance: &InstanceTarget) -> Result<&'a Node, GraphError> {
    instance.id().map_or_else(
      || Err(GraphError::node_not_found(instance)),
      |id| self.find(id).ok_or(GraphError::node_not_found(id)),
    )
  }
}

pub(crate) trait ExpressionWalker {
  fn count(&self) -> usize;
}

impl ExpressionWalker for &[FlowExpression] {
  fn count(&self) -> usize {
    self.iter().map(|e| e.count()).sum()
  }
}

impl ExpressionWalker for &FlowExpression {
  fn count(&self) -> usize {
    match self {
      FlowExpression::BlockExpression(block) => block.inner().count(),
      FlowExpression::ConnectionExpression(_) => 1,
    }
  }
}

pub(super) fn ensure_added(
  schematic: &mut Schematic,
  instance: &mut InstanceTarget,
  handlers: &HandlerMap,
  config: (LiquidOperationConfig, Option<ExecutionSettings>),
  inline_id: &mut usize,
) -> Result<(), GraphError> {
  match instance {
    InstanceTarget::Path { path, id } => {
      let id = match id {
        TargetId::Named(id) | TargetId::Generated(id) => id,
        TargetId::None => {
          debug!(?instance, "received InstanceTarget::Path with no id, skipping..");
          return Ok(());
        }
      };
      let (component_id, op) = path.split_once("::").unwrap(); // unwrap OK if we come from a parsed config.
      debug!(%id,component=component_id,operation=op,"schematic:add_node");
      let (config, settings) = config;
      let op_settings = OperationSettings::new(config, settings);

      let node = schematic.add_and_get_mut(id, NodeReference::new(component_id, op), op_settings);
      decorate(component_id, op, node, handlers)?;
    }
    InstanceTarget::Null(id) => {
      if id.is_none() {
        *inline_id += 1;
        let id_str = format!("drop_{}", inline_id);
        id.replace(id_str.clone());
        let index = schematic.add_external(
          &id_str,
          NodeReference::new(NullComponent::ID, "drop"),
          Default::default(),
        );
        let node = schematic.get_mut(index).unwrap();
        NullComponent::decorate(node).map_err(|e| GraphError::config(id_str, e))?;
      }
    }
    _ => {}
  }

  Ok(())
}

pub(super) fn decorate(
  component: &str,
  operation: &str,
  node: &mut Node,
  handlers: &HandlerMap,
) -> Result<(), GraphError> {
  if component == components::core::CoreComponent::ID {
    match operation {
      components::core::pluck::Op::ID => components::core::pluck::Op::decorate(node),
      components::core::collect::Op::ID => components::core::collect::Op::decorate(node),
      components::core::merge::Op::ID => components::core::merge::Op::decorate(node),
      components::core::sender::Op::ID => components::core::sender::Op::decorate(node),
      components::core::switch::Op::ID => components::core::switch::Op::decorate(node),
      _ => {
        panic!("unhandled core component operation: {}", operation);
      }
    }
    .map_err(|e| GraphError::core_operation(operation, e))?;
    return Ok(());
  } else if component == components::self_component::SelfComponent::ID {
    debug!("skipping {} component at this stage", component);
    return Ok(());
  } else if component == components::null::NullComponent::ID {
    debug!("skipping {} component at this stage", component);
    return Ok(());
  }
  let opsig = handlers
    .get_op_signature(component, operation)
    .ok_or_else(|| GraphError::missing_operation(component, operation, &handlers.get_op_list(component)))?;

  for input in opsig.inputs() {
    debug!(
      id = &node.name,
      port = input.name(),
      component = component,
      operation = operation,
      "input:add",
    );
    node.add_input(input.name());
  }
  for output in opsig.outputs() {
    debug!(
      id = &node.name,
      port = output.name(),
      component = component,
      operation = operation,
      "output:add",
    );
    node.add_output(output.name());
  }
  Ok(())
}
