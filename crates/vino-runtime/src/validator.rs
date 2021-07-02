use crate::dev::prelude::*;
use crate::error::ValidationError;
use crate::schematic_model::Connection;

type ValidationResult<T> = std::result::Result<T, ValidationError>;
pub(crate) struct Validator<'a> {
  model: &'a SchematicModel,
  omit_namespaces: Vec<String>,
}

fn should_omit(ns: &str, list: &[String]) -> bool {
  let omit = list.iter().find(|name| name == &ns);
  if omit.is_some() {
    trace!("Omitting {}", ns);
  }
  omit.is_some()
}

impl<'a> Validator<'a> {
  pub(crate) fn new(model: &'a SchematicModel, omit_namespaces: Vec<String>) -> Self {
    Validator {
      model,
      omit_namespaces,
    }
  }
  pub(crate) fn validate_early_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = Validator::new(model, vec!["self".to_owned()]);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_early_schematic_inputs(),
      validator.assert_early_schematic_outputs(),
      validator.assert_early_qualified_names(),
      validator.assert_no_dangling_references(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::EarlyError(name, results))
    }
  }
  pub(crate) fn validate_late_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = Validator::new(model, vec!["self".to_owned()]);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_component_models(),
      validator.assert_ports_used(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::PostInitError(name, results))
    }
  }
  pub(crate) fn validate_final_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = Validator::new(model, vec![]);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_component_models(),
      validator.assert_ports_used(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::PostInitError(name, results))
    }
  }
  pub(crate) fn validate(model: &'a SchematicModel) -> std::result::Result<(), ValidationError> {
    Self::validate_early_errors(model)?;
    Self::validate_late_errors(model)
  }
  fn assert_no_dangling_references(&self) -> ValidationResult<()> {
    let dangling: Vec<String> = self
      .model
      .get_connections()
      .iter()
      .flat_map(|conn| {
        let from = self.model.get_component_definition(&conn.from.reference);
        let to = self.model.get_component_definition(&conn.to.reference);
        let mut none = vec![];
        if from.is_none() && conn.from.reference != SCHEMATIC_INPUT {
          none.push(Some(conn.from.reference.clone()));
        }
        if to.is_none() && conn.to.reference != SCHEMATIC_OUTPUT {
          none.push(Some(conn.to.reference.clone()));
        }
        none
      })
      .flatten()
      .collect();
    if dangling.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::DanglingReference(dangling))
    }
  }
  fn assert_ports_used(&self) -> ValidationResult<()> {
    let errors: Vec<ValidationError> = self
      .model
      .get_connections()
      .iter()
      .flat_map(|connection| {
        let mut validations = vec![];
        if connection.from.reference != SCHEMATIC_INPUT {
          let r = &connection.from.reference;
          match self.model.get_component_model_by_ref(r) {
            Some(from) => validations.push(is_valid_output(connection, &from)),
            None => {
              if !self.should_omit_ref(r) {
                validations.push(Err(ValidationError::MissingComponentModels(
                  vec![r.clone()],
                )));
              }
            }
          };
        }
        if connection.to.reference != SCHEMATIC_OUTPUT {
          let r = &connection.to.reference;

          match self.model.get_component_model_by_ref(r) {
            Some(to) => validations.push(is_valid_input(connection, &to)),
            None => {
              if !self.should_omit_ref(r) {
                validations.push(Err(ValidationError::MissingComponentModels(
                  vec![r.clone()],
                )));
              }
            }
          }
        }
        validations
      })
      .filter_map(ValidationResult::err)
      .collect();

    if errors.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::InvalidConnections(errors))
    }
  }
  fn should_omit_ref(&self, reference: &str) -> bool {
    let option = self.model.get_component_definition(reference);
    option.map_or(false, |def| {
      should_omit(&def.namespace, &self.omit_namespaces)
    })
  }
  fn should_omit_def(&self, def: &ComponentDefinition) -> bool {
    should_omit(&def.namespace, &self.omit_namespaces)
  }

  fn assert_component_models(&self) -> ValidationResult<()> {
    let missing_components: Vec<String> = self
      .model
      .get_references()
      .filter_map(|r| {
        let def = self.model.get_component_definition(r).unwrap();

        let model = self.model.get_component_model(&def.id);
        (model.is_none() && !self.should_omit_def(&def))
          .then(|| format!("{}=>{}", r.clone(), def.id))
      })
      .collect();

    if missing_components.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::MissingComponentModels(missing_components))
    }
  }
  fn assert_early_schematic_outputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_outputs();
    if ports.is_empty() {
      Err(ValidationError::NoOutputs)
    } else {
      Ok(())
    }
  }
  fn assert_early_schematic_inputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_inputs();
    if ports.is_empty() {
      Err(ValidationError::NoInputs)
    } else {
      Ok(())
    }
  }
  fn assert_early_qualified_names(&self) -> ValidationResult<()> {
    let component_definitions = self.model.get_component_definitions();
    let mut errors = vec![];
    for def in component_definitions {
      if parse_namespace(&def.id).is_err() {
        errors.push(def.id.clone());
      }
    }
    if errors.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::NotFullyQualified(errors))
    }
  }
}

fn is_valid_input(connection: &Connection, to: &ComponentModel) -> ValidationResult<()> {
  let to_port = &connection.to;
  let found_to_port = to.inputs.iter().find(|port| port.name == to_port.name);

  if found_to_port.is_none() {
    Err(ValidationError::InvalidInputPort(
      to_port.clone(),
      connection.clone(),
      to.inputs.clone(),
    ))
  } else {
    Ok(())
  }
}
fn is_valid_output(connection: &Connection, from: &ComponentModel) -> ValidationResult<()> {
  let from_port = &connection.from;
  let found_from_port = from.outputs.iter().find(|port| port.name == from_port.name);

  if found_from_port.is_none() {
    Err(ValidationError::InvalidOutputPort(
      from_port.clone(),
      connection.clone(),
      from.outputs.clone(),
    ))
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use vino_provider::ComponentSignature;
  use vino_rpc::{
    PortSignature,
    ProviderSignature,
  };

  use super::*;
  use crate::test::prelude::*;

  #[test_env_log::test]
  fn test_validate_early_errors() -> Result<()> {
    let def = load_network_manifest("./manifests/native-component.yaml")?;
    let model = SchematicModel::new(def.schematics[0].clone());

    Validator::validate_early_errors(&model)?;
    Ok(())
  }

  #[test_env_log::test]
  fn test_invalid_ports() -> Result<()> {
    let def = load_network_manifest("./manifests/invalid-bad-ports.yaml")?;
    let mut model = SchematicModel::new(def.schematics[0].clone());
    let expected_inputs = vec![PortSignature {
      name: "input".to_owned(),
      type_string: "string".to_owned(),
    }];
    let expected_outputs = vec![PortSignature {
      name: "output".to_owned(),
      type_string: "bytes".to_owned(),
    }];
    let provider = ProviderModel {
      namespace: "vino".to_owned(),
      components: hashmap! {
        "log".to_owned() => ComponentModel {
          namespace: "test-namespace".to_owned(),
          name: "log".to_owned(),
          inputs: expected_inputs.clone(),
          outputs: expected_outputs.clone(),
        }
      },
    };
    model.commit_providers(vec![provider]);
    let result = Validator::validate_late_errors(&model);
    let first = &model.get_connections()[0];
    let second = &model.get_connections()[1];
    let expected = ValidationError::PostInitError(
      model.get_name(),
      vec![ValidationError::InvalidConnections(vec![
        ValidationError::InvalidInputPort(first.to.clone(), first.clone(), expected_inputs),
        ValidationError::InvalidOutputPort(second.from.clone(), second.clone(), expected_outputs),
      ])],
    );
    equals!(result, Err(expected));
    Ok(())
  }

  #[test_env_log::test]
  fn test_self() -> Result<()> {
    // the "self" namespace can't be validated until the non-self parts of every schematic are complete;
    let def = load_network_manifest("./manifests/reference-self.yaml")?;
    let mut model = SchematicModel::new(def.schematics[0].clone());
    let expected_inputs = vec![PortSignature {
      name: "input".to_owned(),
      type_string: "string".to_owned(),
    }];
    let expected_outputs = vec![PortSignature {
      name: "output".to_owned(),
      type_string: "bytes".to_owned(),
    }];
    let provider = ProviderModel {
      namespace: "vino".to_owned(),
      components: hashmap! {
        "log".to_owned() => ComponentModel {
          namespace: "test-namespace".to_owned(),
          name: "log".to_owned(),
          inputs: expected_inputs.clone(),
          outputs: expected_outputs.clone(),
        }
      },
    };
    model.commit_providers(vec![provider]);
    let result = Validator::validate_early_errors(&model);
    equals!(result, Ok(()));
    let result = Validator::validate_late_errors(&model);
    equals!(result, Ok(()));
    let provider = ProviderModel {
      namespace: "self".to_owned(),
      components: hashmap! {
        "child".to_owned() => ComponentModel {
          namespace: "child_ref".to_owned(),
          name: "child".to_owned(),
          inputs: vec![PortSignature {
            name: "child_input".to_owned(),
            type_string: "string".to_owned(),
          }],
          outputs: vec![PortSignature {
            name: "child_output".to_owned(),
            type_string: "bytes".to_owned(),
          }],
        }
      },
    };
    model.commit_providers(vec![provider]);
    let result = Validator::validate_final_errors(&model);
    equals!(result, Ok(()));

    Ok(())
  }

  #[test_env_log::test]
  fn test_connections() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    equals!(model.get_name(), schematic_name);

    let upstream = model
      .get_upstream(&PortReference::new("logger".to_owned(), "input".to_owned()))
      .unwrap();
    equals!(upstream.reference, SCHEMATIC_INPUT);
    equals!(upstream.name, "input");

    Ok(())
  }

  #[test_env_log::test]
  fn test_dangling_refs() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "dangling1".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    equals!(model.get_name(), schematic_name);
    let result = Validator::validate_early_errors(&model);
    equals!(
      result,
      Err(ValidationError::EarlyError(
        "Test".to_owned(),
        vec![
          ValidationError::NoInputs,
          ValidationError::DanglingReference(vec!["dangling1".to_owned()]),
        ]
      ))
    );

    Ok(())
  }

  #[test_env_log::test]
  fn test_missing_models() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    let model = SchematicModel::new(schematic_def);
    let result = Validator::validate_late_errors(&model);
    equals!(
      result,
      Err(ValidationError::PostInitError(
        "Test".to_owned(),
        vec![ValidationError::MissingComponentModels(vec![
          "logger=>test-namespace::log".to_owned()
        ]),]
      ))
    );
    println!("{:?}", model);

    Ok(())
  }

  #[test_env_log::test]
  fn test_finish_initialization() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });
    let mut model = SchematicModel::new(schematic_def);
    let provider = ProviderModel {
      namespace: "test-namespace".to_owned(),
      components: hashmap! {
        "log".to_owned() => ComponentModel {
          namespace: "test-namespace".to_owned(),
          name: "log".to_owned(),
          inputs: vec![PortSignature{name: "input".to_owned(), type_string: "string".to_owned()}],
          outputs: vec![PortSignature{name: "output".to_owned(), type_string: "bytes".to_owned()}],
        }
      },
    };
    model.commit_providers(vec![provider]);
    let result = Validator::validate(&model);
    equals!(result, Ok(()));
    model.finish_initialization()?;
    let schematic_inputs = model.get_schematic_input_signatures()?;
    equals!(
      schematic_inputs,
      &vec![PortSignature {
        name: "input".to_owned(),
        type_string: "string".to_owned()
      }]
    );
    let schematic_outputs = model.get_schematic_output_signatures()?;
    equals!(
      schematic_outputs,
      &vec![PortSignature {
        name: "output".to_owned(),
        type_string: "bytes".to_owned()
      }]
    );
    let provider_sigs = model.get_provider_signatures()?;
    equals!(provider_sigs.len(), 1);
    equals!(
      provider_sigs[0],
      ProviderSignature {
        name: "test-namespace".to_owned(),
        components: vec![ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![PortSignature {
            name: "input".to_owned(),
            type_string: "string".to_owned()
          }],
          outputs: vec![PortSignature {
            name: "output".to_owned(),
            type_string: "bytes".to_owned()
          }]
        }]
      }
    );

    Ok(())
  }
}
