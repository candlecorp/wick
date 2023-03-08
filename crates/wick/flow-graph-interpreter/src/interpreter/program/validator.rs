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
  #[instrument(skip_all, name = "validate-external")]
  fn validate_external_components(&self, program: &Program) -> Result {
    let mut errors = Vec::new();
    let state = &program.state();

    let collections = &state.collections;
    for schematic in state.network.schematics() {
      for component in schematic.nodes() {
        let mut validation_errors = Vec::new();

        if let NodeKind::External(reference) = component.kind() {
          let collection = collections.get(reference.namespace());
          if collection.is_none() {
            validation_errors.push(ValidationError::MissingCollection(reference.namespace().to_owned()));
            continue;
          }
          let collection = collection.unwrap();

          let id = get_id(
            reference.namespace(),
            reference.name(),
            schematic.name(),
            component.id(),
          );

          let component_def = collection.operations.get(&id);

          if component_def.is_none() {
            validation_errors.push(ValidationError::MissingOperation {
              namespace: reference.namespace().to_owned(),
              name: id.clone(),
            });
            continue;
          }

          let component_def = component_def.unwrap();
          for port in component.inputs() {
            let port_def = component_def.inputs.get(port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                component: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
              });
              continue;
            }
          }

          for port in component.outputs() {
            let port_def = component_def.outputs.get(port.name());
            if port_def.is_none() {
              validation_errors.push(ValidationError::InvalidPort {
                port: port.name().to_owned(),
                component: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
              });
              continue;
            }
          }

          for name in component_def.inputs.inner().keys() {
            let port = component.find_input(name);
            if port.is_none() {
              validation_errors.push(ValidationError::MissingConnection {
                port: name.clone(),
                operation: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
              });
              continue;
            }
          }

          for name in component_def.outputs.inner().keys() {
            let port = component.find_output(name);
            if port.is_none() {
              let cref: Reference = component.cref().into();
              if cref.is_core_operation(CORE_ID_SENDER) {
                validation_errors.push(ValidationError::UnusedSender(component.id().to_owned()));
              }

              validation_errors.push(ValidationError::UnusedOutput {
                port: name.clone(),
                component: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
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
