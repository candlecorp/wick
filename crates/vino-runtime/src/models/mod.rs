pub(crate) mod component_model;
pub(crate) mod network_model;
pub(crate) mod provider_model;
pub(crate) mod schematic_model;
pub(crate) mod validator;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
pub(crate) use schematic_model::SchematicModel;

use crate::dev::prelude::*;

pub(crate) mod error;
pub(crate) use error::*;

pub(crate) type SharedModel = Arc<RwLock<SchematicModel>>;

pub(crate) fn get_outputs(model: &SharedModel, instance: &str) -> Vec<ConnectionTargetDefinition> {
  model.read().get_outputs(instance)
}

pub(crate) fn get_port_connections(
  model: &SharedModel,
  port: &ConnectionTargetDefinition,
) -> Vec<ConnectionDefinition> {
  model.read().get_port_connections(port).cloned().collect()
}

pub(crate) fn get_downstream_connections(
  model: &SharedModel,
  instance: &str,
) -> Vec<ConnectionDefinition> {
  model
    .read()
    .get_downstream_connections(instance)
    .cloned()
    .collect()
}

pub(crate) fn get_component_definition(
  model: &SharedModel,
  instance: &str,
) -> Result<ComponentDefinition, SchematicError> {
  model
    .read()
    .get_component_definition(instance)
    .ok_or_else(|| SchematicError::InstanceNotFound(instance.to_owned()))
}

pub(crate) fn get_incoming_ports(model: &SharedModel) -> HashMap<String, RawPorts> {
  model.read().get_raw_ports().clone()
}
