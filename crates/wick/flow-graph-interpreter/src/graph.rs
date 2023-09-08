mod error;
mod helpers;
mod operation_settings;
pub(crate) mod types;
use std::collections::HashMap;

pub use error::Error as GraphError;
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
use wick_config::config::{ComponentImplementation, ExecutionSettings, FlowOperation};
use wick_packet::RuntimeConfig;

use self::helpers::{ensure_added, ParseHelper};
pub(crate) use self::operation_settings::{LiquidOperationConfig, OperationSettings};
use crate::interpreter::components::core;
use crate::HandlerMap;

pub(crate) trait NodeDecorator {
  fn decorate(node: &mut Node) -> Result<(), String>;
}

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
}

fn register_operation(
  mut scope: Vec<String>,
  network: &mut Network,
  flow: &mut FlowOperation,
  handlers: &HandlerMap,
  op_config_base: &LiquidOperationConfig,
) -> Result<(), GraphError> {
  scope.push(flow.name().to_owned());

  for flow in flow.flows_mut() {
    let scope = scope.clone();
    register_operation(scope, network, flow, handlers, op_config_base)?;
  }
  let name = scope.join("::");
  let mut schematic = Schematic::new(name, Default::default(), Default::default());
  let mut ids = flow.instances().keys().cloned().collect::<Vec<_>>();
  ids.sort();

  for name in ids {
    let def = flow.instances().get(&name).unwrap();
    debug!(%name, config=?def.data(),settings=?def.settings(), "registering operation");
    let mut op_config = op_config_base.clone();
    op_config.set_template(def.data().cloned());

    let node = schematic.add_and_get_mut(
      name,
      NodeReference::new(def.component_id(), def.name()),
      OperationSettings::new(op_config.clone(), def.settings().cloned()),
    );
    helpers::decorate(def.component_id(), def.name(), node, handlers)?;
  }

  expand_until_done(&mut schematic, flow, handlers, op_config_base, expand_expressions)?;

  for expression in flow.expressions() {
    process_flow_expression(&mut schematic, expression, handlers)?;
  }

  network.add_schematic(schematic);
  Ok(())
}

fn process_flow_expression(
  schematic: &mut Schematic,
  expr: &FlowExpression,
  handlers: &HandlerMap,
) -> Result<(), GraphError> {
  match expr {
    FlowExpression::ConnectionExpression(expr) => process_connection_expression(schematic, expr, handlers)?,
    FlowExpression::BlockExpression(expr) => {
      for expr in expr.iter() {
        process_flow_expression(schematic, expr, handlers)?;
      }
    }
  }
  Ok(())
}

fn process_connection_expression(
  schematic: &mut Schematic,
  expr: &ConnectionExpression,
  _handlers: &HandlerMap,
) -> Result<(), GraphError> {
  let from = expr.from();
  let to = expr.to();
  assert!(
    to.port().name().is_some(),
    "Missing downstream port for expr: {:?}",
    expr
  );
  let to_port = schematic
    .find_mut(to.instance().id().unwrap())
    .map(|component| component.add_input(to.port().name().unwrap()));

  if to_port.is_none() {
    error!("missing downstream: instance {:?}", to);
    return Err(GraphError::missing_downstream(to.instance().id().unwrap()));
  }
  let to_port = to_port.unwrap();

  if let Some(component) = schematic.find_mut(from.instance().id().unwrap()) {
    let from_port = component.add_output(from.port().name().unwrap());
    trace!(
      ?from_port,
      from = %expr.from(),
      ?to_port,
      to = %expr.to(),
      "graph:connecting"
    );
    schematic.connect(from_port, to_port, Default::default())?;
  } else {
    panic!("Can't find component {}", from.instance());
  }
  Ok(())
}

#[allow(trivial_casts)]
fn expand_until_done(
  schematic: &mut Schematic,
  expressions: &mut FlowOperation,
  handlers: &HandlerMap,
  config: &LiquidOperationConfig,
  func: fn(
    &mut Schematic,
    &mut FlowOperation,
    &HandlerMap,
    &LiquidOperationConfig,
    &mut usize,
  ) -> Result<ExpandResult, GraphError>,
) -> Result<(), GraphError> {
  let mut id_index = 0;
  loop {
    let result = func(schematic, expressions, handlers, config, &mut id_index)?;

    if result == ExpandResult::Done {
      break;
    }
  }
  Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ExpandResult {
  Done,
  Continue,
}

impl ExpandResult {
  fn update(self, next: ExpandResult) -> Self {
    if self == ExpandResult::Continue {
      self
    } else {
      next
    }
  }
}

#[allow(clippy::option_if_let_else)]
fn expand_expressions(
  schematic: &mut Schematic,
  flow: &mut FlowOperation,
  handlers: &HandlerMap,
  config: &LiquidOperationConfig,
  inline_id: &mut usize,
) -> Result<ExpandResult, GraphError> {
  let result = ExpandResult::Done;

  let config_map = flow
    .instances()
    .iter()
    .map(|(k, v)| {
      let mut base = config.clone();
      base.set_template(v.data().cloned());

      Ok::<_, GraphError>((k.clone(), (base, v.settings().cloned())))
    })
    .collect::<Result<HashMap<_, _>, _>>()?;
  add_nodes_to_schematic(schematic, flow.expressions_mut(), handlers, &config_map, inline_id)?;
  let result = result.update(expand_port_paths(schematic, flow.expressions_mut())?);
  let result = result.update(expand_defaulted_ports(schematic, flow.expressions_mut())?);
  Ok(result)
}

fn add_nodes_to_schematic(
  schem: &mut Schematic,
  flow: &mut [FlowExpression],
  handlers: &HandlerMap,
  config_map: &HashMap<String, (LiquidOperationConfig, Option<ExecutionSettings>)>,
  id_index: &mut usize,
) -> Result<(), GraphError> {
  for (_i, expression) in flow.iter_mut().enumerate() {
    match expression {
      FlowExpression::ConnectionExpression(conn) => {
        let config = conn
          .from()
          .instance()
          .id()
          .and_then(|id| config_map.get(id).cloned())
          .unwrap_or((LiquidOperationConfig::default(), None));

        ensure_added(schem, conn.from_mut().instance_mut(), handlers, config, id_index)?;

        let config = conn
          .to()
          .instance()
          .id()
          .and_then(|id| config_map.get(id).cloned())
          .unwrap_or((LiquidOperationConfig::default(), None));

        ensure_added(schem, conn.to_mut().instance_mut(), handlers, config, id_index)?;
      }
      FlowExpression::BlockExpression(expressions) => {
        add_nodes_to_schematic(schem, expressions.inner_mut(), handlers, config_map, id_index)?;
      }
    }
  }

  Ok(())
}

fn connection(
  from: (InstanceTarget, impl Into<InstancePort>),
  to: (InstanceTarget, impl Into<InstancePort>),
) -> FlowExpression {
  FlowExpression::connection(ConnectionExpression::new(
    ConnectionTargetExpression::new(from.0, from.1),
    ConnectionTargetExpression::new(to.0, to.1),
  ))
}

#[allow(clippy::option_if_let_else, clippy::too_many_lines, clippy::cognitive_complexity)]
fn expand_defaulted_ports(
  schematic: &mut Schematic,
  expressions: &mut [FlowExpression],
) -> Result<ExpandResult, GraphError> {
  let mut result = ExpandResult::Done;
  for (_i, expression) in expressions.iter_mut().enumerate() {
    match expression {
      FlowExpression::ConnectionExpression(expr) => {
        let (from, to) = expr.clone().into_parts();
        let (from_inst, from_port, _) = from.into_parts();
        let (to_inst, to_port, _) = to.into_parts();
        match (from_port, to_port) {
          (InstancePort::None, InstancePort::None) => {
            let from_node = schematic.get_node(&from_inst)?;
            let to_node = schematic.get_node(&to_inst)?;
            let from_node_ports = from_node.outputs();
            let to_node_ports = to_node.inputs();
            debug!(
              from = %from_inst, from_ports = ?from_node_ports, to = %to_inst, to_ports = ?to_node_ports,
              "graph:inferring ports for both up and down"
            );
            if from_node_ports.is_empty() && to_node_ports.is_empty() {
              // can't do anything yet.
              continue;
            }

            // If there's only one port on each side, connect them.
            if from_node_ports.len() == 1 && to_node_ports.len() == 1 {
              let from_port = from_node_ports[0].name();
              let to_port = to_node_ports[0].name();
              debug!(from = %from_inst, from_port,to = %to_inst, to_port, reason="unary", "graph:inferred ports");
              expression.replace(connection((from_inst, from_port), (to_inst, to_port)));
              result = ExpandResult::Continue;
              continue;
            }

            let mut new_connections = Vec::new();
            // if either side is a schematic input/output node, adopt the names of all ports we're pointing to.
            if matches!(from_inst, InstanceTarget::Input | InstanceTarget::Default) {
              for port in to_node_ports {
                let port_name = port.name();
                debug!(from = %from_inst, from_port=port_name,to = %to_inst, to_port=port_name, reason="upstream_default", "graph:inferred ports");
                new_connections.push(connection((from_inst.clone(), port_name), (to_inst.clone(), port_name)));
              }
            } else if matches!(to_inst, InstanceTarget::Output | InstanceTarget::Default) {
              for port in from_node_ports {
                let port_name = port.name();
                debug!(from = %from_inst, from_port=port_name,to = %to_inst, to_port=port_name, reason="downstream_default", "graph:inferred ports");
                new_connections.push(connection((from_inst.clone(), port_name), (to_inst.clone(), port_name)));
              }
            } else {
              for port in from_node_ports {
                if !to_node_ports.contains(port) && !matches!(to_inst, InstanceTarget::Output | InstanceTarget::Default)
                {
                  return Err(GraphError::port_inference_down(
                    &from_inst,
                    port.name(),
                    to_inst,
                    to_node_ports,
                  ));
                }
                let port_name = port.name();
                debug!(from = %from_inst, from_port=port_name,to = %to_inst, to_port=port_name, reason="all_downstream", "graph:inferred ports");
                new_connections.push(connection(
                  (from_inst.clone(), port.name()),
                  (to_inst.clone(), port.name()),
                ));
              }
            }

            assert!(!new_connections.is_empty(), "unhandled case for port inference");
            result = ExpandResult::Continue;
            expression.replace(FlowExpression::block(BlockExpression::new(new_connections)));
          }
          (InstancePort::None, to_port) => {
            let port_name = to_port.name().unwrap();
            let from_node = schematic.get_node(&from_inst)?;
            let ports = from_node.outputs();
            debug!(
              from = %from_inst, from_ports = ?ports, to = %to_inst,
              "graph:inferring ports for upstream"
            );
            // if we're at a schematic input node, adopt the name of what we're pointing to.
            if matches!(from_inst, InstanceTarget::Input | InstanceTarget::Default) {
              expression.replace(connection((from_inst, port_name), (to_inst, to_port.clone())));
              result = ExpandResult::Continue;
              continue;
            }
            if ports.len() == 1 {
              expression.replace(connection((from_inst, ports[0].name()), (to_inst, to_port.clone())));
              result = ExpandResult::Continue;
              continue;
            }

            if !ports.iter().any(|p| p.name() == port_name) {
              return Err(GraphError::port_inference_up(&to_inst, port_name, from_inst, ports));
            }

            result = ExpandResult::Continue;
            expression.replace(connection((from_inst, port_name), (to_inst, to_port.clone())));
          }
          (from_port, InstancePort::None) => {
            let port_name = from_port.name().unwrap();
            let to_node = schematic.get_node(&to_inst)?;
            let ports = to_node.inputs();
            debug!(
              from = %from_inst, to = %to_inst, to_ports = ?ports,
              "graph:inferring ports for downstream"
            );

            // if we're at a schematic input node, adopt the name of what we're pointing to.
            if matches!(to_inst, InstanceTarget::Output | InstanceTarget::Default) {
              expression.replace(connection((from_inst, from_port.clone()), (to_inst, port_name)));
              result = ExpandResult::Continue;
              continue;
            }

            if ports.len() == 1 {
              expression.replace(connection((from_inst, from_port.clone()), (to_inst, ports[0].name())));
              result = ExpandResult::Continue;
              continue;
            }

            if !ports.iter().any(|p| p.name() == port_name) {
              return Err(GraphError::port_inference_down(&from_inst, port_name, to_inst, ports));
            }

            result = ExpandResult::Continue;
            expression.replace(connection((from_inst, from_port.clone()), (to_inst, port_name)));
          }
          _ => continue,
        }
      }
      FlowExpression::BlockExpression(expressions) => {
        result = result.update(expand_defaulted_ports(schematic, expressions.inner_mut())?);
      }
    }
  }
  Ok(result)
}

#[allow(clippy::option_if_let_else)]
fn expand_port_paths(
  schematic: &mut Schematic,
  expressions: &mut [FlowExpression],
) -> Result<ExpandResult, GraphError> {
  let mut result = ExpandResult::Done;
  for (i, expression) in expressions.iter_mut().enumerate() {
    match expression {
      FlowExpression::ConnectionExpression(expr) => {
        let (from, to) = expr.clone().into_parts();
        let (from_inst, from_port, _) = from.into_parts();
        let (to_inst, to_port, _) = to.into_parts();
        if let InstancePort::Path(name, parts) = from_port {
          let id = format!("{}_pluck_{}_{}_[{}]", schematic.name(), i, name, parts.join(","));
          let config = HashMap::from([(
            "path".to_owned(),
            Value::Array(parts.into_iter().map(Value::String).collect()),
          )]);

          let node = schematic.add_and_get_mut(
            &id,
            NodeReference::new("core", "pluck"),
            OperationSettings::new(Some(RuntimeConfig::from(config)).into(), None),
          );
          core::pluck::Op::decorate(node).map_err(|e| GraphError::config(id.clone(), e))?;

          expression.replace(FlowExpression::block(BlockExpression::new(vec![
            connection((from_inst, &name), (InstanceTarget::named(&id), InstancePort::None)),
            connection((InstanceTarget::named(&id), InstancePort::None), (to_inst, to_port)),
          ])));
          result = ExpandResult::Continue;
        }
      }
      FlowExpression::BlockExpression(expressions) => {
        result = result.update(expand_port_paths(schematic, expressions.inner_mut())?);
      }
    }
  }
  Ok(result)
}

pub fn from_def(
  manifest: &mut wick_config::config::ComponentConfiguration,
  handlers: &HandlerMap,
) -> Result<Network, GraphError> {
  let mut network = Network::new(
    manifest.name().cloned().unwrap_or_default(),
    OperationSettings::new(manifest.root_config().cloned().into(), None),
  );

  let mut op_config_base = LiquidOperationConfig::default();
  op_config_base.set_root(manifest.root_config().cloned());

  if let ComponentImplementation::Composite(composite) = manifest.component_mut() {
    for flow in composite.operations_mut() {
      register_operation(vec![], &mut network, flow, handlers, &op_config_base)?;
    }
  }

  #[cfg(debug_assertions)]
  {
    let names: Vec<_> = network.schematics().iter().map(|s| s.name()).collect();
    trace!(nodes=?names,"graph:nodes");
    for schematic in network.schematics() {
      let schem_name = &schematic.name();
      for node in schematic.nodes() {
        let name = &node.name;
        let inputs = node.inputs().iter().map(|n| n.name()).collect::<Vec<_>>();
        let outputs = node.outputs().iter().map(|n| n.name()).collect::<Vec<_>>();
        trace!(schematic = schem_name, node = name, ?inputs, ?outputs, data=?node.data(), "graph:node");
      }
    }
  }

  Ok(network)
}
