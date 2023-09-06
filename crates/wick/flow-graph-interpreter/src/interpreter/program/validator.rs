use flow_graph::iterators::{SchematicWalker, WalkDirection};
use flow_graph::NodeKind;

use self::error::{OperationInvalid, ValidationError};
use super::Program;
use crate::interpreter::components::reconcile_op_id;

pub(crate) mod error;

type Result = std::result::Result<(), Vec<OperationInvalid>>;

pub(crate) struct Validator {}

impl Validator {
  #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
  fn validate_external_components(&self, program: &Program) -> Result {
    let mut errors = Vec::new();
    let state = &program.state();

    let components = &state.components;
    for schematic in state.network.schematics() {
      let schematic_name = schematic.name();
      for node in schematic.nodes() {
        let mut validation_errors = Vec::new();

        if let NodeKind::External(reference) = node.kind() {
          let component = components.get(reference.component_id());

          if component.is_none() {
            error!("missing component: {}", reference.component_id());
            validation_errors.push(ValidationError::MissingComponent(reference.component_id().to_owned()));
            continue;
          }
          let component = component.unwrap();

          let id = reconcile_op_id(reference.component_id(), reference.name(), schematic_name, node.id());

          let operation_sig = component.get_operation(&id);

          if operation_sig.is_none() {
            validation_errors.push(ValidationError::MissingOperation {
              component: reference.component_id().to_owned(),
              name: id.clone(),
            });
            continue;
          }

          let operation_sig = operation_sig.unwrap();
          for port in node.inputs() {
            let port_def = operation_sig.inputs.iter().find(|p| p.name == port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                id: node.name.clone(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }

          for port in node.outputs() {
            let port_def = operation_sig.outputs.iter().find(|p| p.name == port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                id: node.name.clone(),
                operation: reference.name().to_owned(),
                component: reference.component_id().to_owned(),
              });
              continue;
            }
          }

          for field in &operation_sig.inputs {
            match node.find_input(&field.name) {
              Some(port) => {
                let mut walker = SchematicWalker::from_port(schematic, port.detached(), WalkDirection::Up);
                let next = walker.nth(1);

                if next.is_none() {
                  validation_errors.push(ValidationError::MissingConnection {
                    port: field.name.clone(),
                    id: node.name.clone(),
                    operation: reference.name().to_owned(),
                    component: reference.component_id().to_owned(),
                  });
                  continue;
                }
              }
              None => {
                validation_errors.push(ValidationError::MissingPort {
                  port: field.name.clone(),
                  id: node.name.clone(),
                  operation: reference.name().to_owned(),
                  component: reference.component_id().to_owned(),
                });
                continue;
              }
            };
          }

          for field in &operation_sig.outputs {
            match node.find_output(&field.name) {
              Some(port) => {
                let mut walker = SchematicWalker::from_port(schematic, port.detached(), WalkDirection::Down);
                let next = walker.nth(1);
                if next.is_none() {
                  debug!(id=node.name.clone(), operation = %reference, port = %field.name, "unhandled downstream connection");
                  // validation_errors.push(ValidationError::UnusedOutput {
                  //   port: field.name.clone(),
                  //   id: node.name.clone(),
                  //   operation: reference.name().to_owned(),
                  //   component: reference.component_id().to_owned(),
                  // });
                  // continue;
                }
              }
              None => {
                validation_errors.push(ValidationError::MissingPort {
                  port: field.name.clone(),
                  id: node.name.clone(),
                  operation: reference.name().to_owned(),
                  component: reference.component_id().to_owned(),
                });
                continue;
              }
            };
          }
        }

        if !validation_errors.is_empty() {
          errors.push(OperationInvalid::new(schematic.name().to_owned(), validation_errors));
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
