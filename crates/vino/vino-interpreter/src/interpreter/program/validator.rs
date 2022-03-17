use vino_schematic_graph::ComponentKind;
use vino_types::MapWrapper;

use self::error::ValidationError;
use super::Program;

pub(crate) mod error;

type Result = std::result::Result<(), Vec<ValidationError>>;

pub(crate) struct Validator {}

impl Validator {
  fn validate_external_components(&self, program: &Program) -> Result {
    let mut errors = Vec::new();
    let state = program.state();

    let components = &state.components;
    for schematic in state.network.schematics() {
      for component in schematic.components() {
        if let ComponentKind::External(reference) = component.kind() {
          let provider = components.get(reference.namespace());
          if provider.is_none() {
            errors.push(ValidationError::MissingProvider(reference.namespace().to_owned()));
            continue;
          }
          let provider = provider.unwrap();
          let component_def = provider.get(reference.name());
          if component_def.is_none() {
            errors.push(ValidationError::MissingComponent {
              name: reference.name().to_owned(),
              namespace: reference.namespace().to_owned(),
            });
            continue;
          }

          let component_def = component_def.unwrap();
          for port in component.inputs() {
            let port_def = component_def.inputs.get(port.name());
            if port_def.is_none() {
              errors.push(ValidationError::InvalidPort {
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
              errors.push(ValidationError::InvalidPort {
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
              errors.push(ValidationError::MissingPort {
                port: name.clone(),
                component: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
              });
              continue;
            }
          }

          for name in component_def.outputs.inner().keys() {
            let port = component.find_output(name);
            if port.is_none() {
              errors.push(ValidationError::MissingPort {
                port: name.clone(),
                component: reference.name().to_owned(),
                namespace: reference.namespace().to_owned(),
              });
              continue;
            }
          }
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
