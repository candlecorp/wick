use flow_graph::NodeKind;

use self::error::{SchematicInvalid, ValidationError};
use super::Program;
use crate::constants::CORE_ID_SENDER;
use crate::graph::Reference;
use crate::interpreter::collections::get_id;

pub(crate) mod error;

type Result = std::result::Result<(), Vec<SchematicInvalid>>;

pub(crate) struct Validator {}

impl Validator {
  fn validate_external_components(&self, program: &Program) -> Result {
    let mut errors = Vec::new();
    let state = &program.state();

    let components = &state.components;
    for schematic in state.network.schematics() {
      for operation in schematic.nodes() {
        let mut validation_errors = Vec::new();

        if let NodeKind::External(reference) = operation.kind() {
          let component = components.get(reference.component_id());

          if component.is_none() {
            error!("Missing component: {}", reference.component_id());
            validation_errors.push(ValidationError::MissingComponent(reference.component_id().to_owned()));
            continue;
          }
          let component = component.unwrap();

          let id = get_id(
            reference.component_id(),
            reference.name(),
            schematic.name(),
            operation.id(),
          );

          let operation_sig = component.operations.iter().find(|op| op.name == id);

          if operation_sig.is_none() {
            validation_errors.push(ValidationError::MissingOperation {
              component: reference.component_id().to_owned(),
              name: id.clone(),
            });
            continue;
          }

          let operation_sig = operation_sig.unwrap();
          for port in operation.inputs() {
            let port_def = operation_sig.inputs.iter().find(|p| p.name == port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }

          for port in operation.outputs() {
            let port_def = operation_sig.outputs.iter().find(|p| p.name == port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }

          for field in &operation_sig.inputs {
            let port = operation.find_input(&field.name);
            if port.is_none() {
              validation_errors.push(ValidationError::MissingConnection {
                port: field.name.clone(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }

          for field in &operation_sig.outputs {
            let port = operation.find_output(&field.name);
            if port.is_none() {
              let cref: Reference = operation.cref().into();
              if cref.is_core_operation(CORE_ID_SENDER) {
                validation_errors.push(ValidationError::UnusedSender(operation.id().to_owned()));
              }

              validation_errors.push(ValidationError::UnusedOutput {
                port: field.name.clone(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }
        }

        if !validation_errors.is_empty() {
          errors.push(SchematicInvalid::new(schematic.name().to_owned(), validation_errors));
        }
      }
    }
    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }
}

pub(crate) fn validate(program: &Program) -> Result {
  let validator = Validator {};
  validator.validate_external_components(program)?;
  Ok(())
}
