use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, Vec<ValidationErrorKind>>;
pub(crate) struct SchematicValidator<'a> {
  model: &'a SchematicModel,
  omit_namespaces: Vec<String>,
}

impl<'a> SchematicValidator<'a> {
  pub(crate) fn new(model: &'a SchematicModel, omit_namespaces: Vec<String>) -> Self {
    SchematicValidator {
      model,
      omit_namespaces,
    }
  }
  pub(crate) fn validate_early_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = SchematicValidator::new(model, vec!["self".to_owned()]);
    let name = model.get_name();
    let results: Vec<ValidationErrorKind> = vec![
      validator.assert_early_schematic_outputs(),
      validator.assert_early_qualified_names(),
      validator.assert_no_dangling_references(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .flatten()
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::new(&name, results))
    }
  }

  pub(crate) fn validate_final_errors(
    model: &'a SchematicModel,
  ) -> std::result::Result<(), ValidationError> {
    let validator = SchematicValidator::new(model, vec![]);
    let name = model.get_name();

    let results: Vec<ValidationErrorKind> = vec![
      validator.assert_component_models(),
      validator.assert_ports_used(),
      validator.assert_inputs_connected(),
    ]
    .into_iter()
    .filter_map(std::result::Result::err)
    .flatten()
    .collect();

    if results.is_empty() {
      Ok(())
    } else {
      Err(ValidationError::new(&name, results))
    }
  }
  pub(crate) fn _validate(model: &'a SchematicModel) -> std::result::Result<(), ValidationError> {
    Self::validate_early_errors(model)
  }
  fn assert_no_dangling_references(&self) -> Result<()> {
    let dangling: Vec<ValidationErrorKind> = self
      .model
      .get_connections()
      .iter()
      .flat_map(|conn| {
        let mut none = vec![];

        let from_instance = conn.from.get_instance();
        let from = self.model.get_component_definition(from_instance);
        if from.is_none() && !conn.has_default() && !conn.from.is_system_upstream() {
          none.push(ValidationErrorKind::DanglingReference(
            from_instance.to_owned(),
          ));
        }

        let to_instance = conn.to.get_instance();
        let to = self.model.get_component_definition(to_instance);
        if to.is_none() && !conn.to.is_system_downstream() {
          none.push(ValidationErrorKind::DanglingReference(
            to_instance.to_owned(),
          ));
        }

        none
      })
      .collect();
    if dangling.is_empty() {
      Ok(())
    } else {
      Err(dangling)
    }
  }

  fn assert_inputs_connected(&self) -> Result<()> {
    let errors: Vec<ValidationErrorKind> = self
      .model
      .get_connections()
      .iter()
      .flat_map(|connection| {
        let mut validations = vec![];
        let instance = connection.to.get_instance();
        let model_opt = self.model.get_component_model_by_instance(instance);
        if let Some((_, model)) = model_opt {
          let mut disconnected = vec![];
          for input in model.inputs().names() {
            let mut upstreams = self.model.get_upstream_connections_by_instance(instance);
            if !upstreams.any(|def| def.to.matches_port(&input)) {
              disconnected.push(input);
            }
          }
          for missing_input in disconnected {
            validations.push(ValidationErrorKind::MissingInputConnection(
              instance.to_owned(),
              missing_input,
            ));
          }
        }
        validations
      })
      .collect();

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn assert_ports_used(&self) -> Result<()> {
    let errors: Vec<ValidationErrorKind> = self
      .model
      .get_connections()
      .iter()
      .flat_map(|connection| {
        let mut validations = vec![];
        let is_schematic_input = connection.from.matches_instance(SCHEMATIC_INPUT);
        let is_sender = connection.from.is_sender();
        if !is_sender && !is_schematic_input {
          let instance = connection.from.get_instance();
          let model_opt = self.model.get_component_model_by_instance(instance);
          if let Some((_, from)) = model_opt {
            validations.push(is_valid_output(connection, &from));
          }
        }
        let is_schematic_output = connection.to.matches_instance(SCHEMATIC_OUTPUT);
        if !is_schematic_output {
          let instance = connection.to.get_instance();
          let model_opt = self.model.get_component_model_by_instance(instance);
          if let Some((_, to)) = model_opt {
            validations.push(is_valid_input(connection, &to));
          }
        }
        validations
      })
      .filter_map(|res| res.err())
      .collect();

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn should_omit_def(&self, def: &ComponentDefinition) -> bool {
    self
      .omit_namespaces
      .iter()
      .any(|name| name == &def.namespace)
  }

  fn assert_component_models(&self) -> Result<()> {
    let missing_components: Vec<ValidationErrorKind> = self
      .model
      .get_instances()
      .filter_map(|r| {
        self.model.get_component_definition(r).map_or(
          Some(ValidationErrorKind::InstanceNotFound(r.clone())),
          |def| {
            let is_allowed_provider = self.model.is_provider_allowed(&def.namespace);
            if !is_allowed_provider {
              Some(ValidationErrorKind::MissingProvider(
                def.id(),
                def.namespace,
              ))
            } else {
              let has_model = self.model.get_component_model(&def.id()).is_some();
              let is_err = !has_model && !self.should_omit_def(&def);
              is_err.then(|| {
                ValidationErrorKind::MissingComponentModel(format!("{} ({})", r.clone(), def.id()))
              })
            }
          },
        )
      })
      .collect();

    if missing_components.is_empty() {
      Ok(())
    } else {
      Err(missing_components)
    }
  }

  fn assert_early_schematic_outputs(&self) -> Result<()> {
    let mut ports = self.model.get_schematic_outputs();
    if ports.next().is_none() {
      Err(vec![ValidationErrorKind::NoOutputs])
    } else {
      Ok(())
    }
  }

  // Validate that the passed port has an upstream that either connects
  // to the schematic input or a port that has a default
  fn _validate_port_has_upstream_input(&self, port: &ConnectionTargetDefinition) -> bool {
    let connection = some_or_bail!(self.model.get_upstream_connection(port), false);

    let upstream_ref = connection.from.get_instance();
    let upstream_connections = self
      .model
      .get_upstream_connections_by_instance(upstream_ref);
    for conn in upstream_connections {
      if self._validate_port_has_upstream_input(&conn.to) {
        return true;
      }
    }
    false
  }

  fn assert_early_qualified_names(&self) -> Result<()> {
    let component_definitions = self.model.get_component_definitions();
    let mut errors = vec![];
    for def in component_definitions {
      if parse_id(&def.id()).is_err() {
        errors.push(ValidationErrorKind::NotFullyQualified(def.id()));
      }
    }
    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }
}

fn is_valid_input(
  connection: &ConnectionDefinition,
  to: &ComponentModel,
) -> std::result::Result<(), ValidationErrorKind> {
  let to_port = &connection.to;
  let found_to_port = to.get_input(to_port.get_port());

  if found_to_port.is_none() {
    Err(ValidationErrorKind::InvalidInputPort(
      to_port.clone(),
      connection.clone(),
      to.inputs().names(),
    ))
  } else {
    Ok(())
  }
}

fn is_valid_output(
  connection: &ConnectionDefinition,
  from: &ComponentModel,
) -> std::result::Result<(), ValidationErrorKind> {
  let from_port = &connection.from;
  let found_from_port = from.get_output(from_port.get_port());

  if found_from_port.is_none() {
    Err(ValidationErrorKind::InvalidOutputPort(
      from_port.clone(),
      connection.clone(),
      from.outputs().names(),
    ))
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryInto;
  use std::str::FromStr;

  use vino_manifest::schematic_definition::SenderData;
  use ConnectionTargetDefinition as Target;

  use super::*;
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  use crate::VINO_V0_NAMESPACE;
  #[test_logger::test]
  fn test_validate_early_errors() -> TestResult<()> {
    let def = load_network_definition("./manifests/v0/native-component.yaml")?;
    let model = SchematicModel::try_from(def.schematics[0].clone())?;

    SchematicValidator::validate_early_errors(&model)?;
    Ok(())
  }

  #[test_logger::test]
  fn test_invalid_ports() -> TestResult<()> {
    let def = load_network_definition("./manifests/v0/errors/invalid-bad-ports.yaml")?;
    let mut model = SchematicModel::try_from(def.schematics[0].clone())?;

    let provider = ProviderModel {
      components: hashmap! {
        "log".to_owned() => ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![("input", "string")].try_into()?,
          outputs: vec![("output", "bytes")].try_into()?
        }.into()
      },
    };
    model.allow_providers(&[VINO_V0_NAMESPACE]);
    model.commit_providers(hashmap! {VINO_V0_NAMESPACE.to_owned()=>provider})?;
    let result = SchematicValidator::validate_final_errors(&model);
    let first = &model.get_connections()[0];
    let second = &model.get_connections()[1];
    let expected = ValidationError::new(
      &model.get_name(),
      vec![
        ValidationErrorKind::InvalidInputPort(
          first.to.clone(),
          first.clone(),
          vec!["input".to_owned()],
        ),
        ValidationErrorKind::InvalidOutputPort(
          second.from.clone(),
          second.clone(),
          vec!["output".to_owned()],
        ),
        ValidationErrorKind::MissingInputConnection("logger".to_owned(), "input".to_owned()),
      ],
    );
    assert_eq!(result, Err(expected));
    Ok(())
  }

  #[test_logger::test]
  fn test_missing_inputs() -> TestResult<()> {
    let def = load_network_definition("./manifests/v0/errors/missing-inputs.yaml")?;
    let mut model = SchematicModel::try_from(def.schematics[0].clone())?;

    let provider = ProviderModel {
      components: hashmap! {
        "add".to_owned() => ComponentSignature {
          name: "add".to_owned(),
          inputs: vec![("left", "u32"),("right", "u32")].try_into()?,
          outputs: vec![("output", "u32")].try_into()?
        }.into()
      },
    };
    model.allow_providers(&[VINO_V0_NAMESPACE]);
    model.commit_providers(hashmap! {VINO_V0_NAMESPACE.to_owned()=>provider})?;
    let result = SchematicValidator::validate_final_errors(&model);
    let expected = ValidationError::new(
      &model.get_name(),
      vec![ValidationErrorKind::MissingInputConnection(
        "add".to_owned(),
        "right".to_owned(),
      )],
    );
    assert_eq!(result, Err(expected));
    Ok(())
  }

  #[test_logger::test]
  fn test_self() -> TestResult<()> {
    // the "self" namespace can't be validated until the non-self parts of every schematic are complete;
    let def = load_network_definition("./manifests/v0/reference-self.yaml")?;
    let mut model = SchematicModel::try_from(def.schematics[0].clone())?;

    let provider = ProviderModel {
      components: hashmap! {
        "log".to_owned() => ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![("input", "string")].try_into()?,
          outputs: vec![("output", "bytes")].try_into()?
        }.into()
      },
    };
    model.commit_providers(hashmap! {"vino".to_owned()=>provider})?;
    let result = SchematicValidator::validate_early_errors(&model);
    assert_eq!(result, Ok(()));
    let provider = ProviderModel {
      components: hashmap! {
        "child".to_owned() => ComponentSignature {
          name: "child".to_owned(),
          inputs: vec![("child_input", "string")].try_into()?,
          outputs: vec![("child_output", "bytes")].try_into()?
        }.into()
      },
    };
    model.commit_providers(hashmap! {"self".to_owned()=>provider})?;
    let result = SchematicValidator::validate_final_errors(&model);
    assert_eq!(result, Ok(()));

    Ok(())
  }

  #[test_logger::test]
  fn test_connections() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push("test-namespace".to_owned());
    schematic_def.instances.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: Target::new(SCHEMATIC_INPUT, "input"),
      to: Target::new("logger", "input"),
      default: None,
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: Target::new("logger", "output"),
      to: Target::new(SCHEMATIC_OUTPUT, "output"),
      default: None,
    });
    let model = SchematicModel::try_from(schematic_def)?;
    assert_eq!(model.get_name(), schematic_name);

    let upstream = model
      .get_upstream(&Target::new("logger".to_owned(), "input".to_owned()))
      .unwrap();
    assert_eq!(upstream.get_instance(), SCHEMATIC_INPUT);
    assert_eq!(upstream.get_port(), "input");

    Ok(())
  }

  #[test_logger::test]
  fn test_dangling_refs() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: Target::new("dangling1", "output"),
      to: Target::new(SCHEMATIC_OUTPUT, "output"),
      default: None,
    });
    let model = SchematicModel::try_from(schematic_def)?;
    assert_eq!(model.get_name(), schematic_name);
    let result = SchematicValidator::validate_early_errors(&model);
    assert_eq!(
      result,
      Err(ValidationError::new(
        schematic_name,
        vec![ValidationErrorKind::DanglingReference(
          "dangling1".to_owned()
        ),]
      ))
    );

    Ok(())
  }

  #[test_logger::test]
  fn test_sender() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.connections.push(ConnectionDefinition {
      from: Target::sender(Some(SenderData::from_str("\"Default string\"")?)),
      to: Target::new(SCHEMATIC_OUTPUT, "output"),
      default: None,
    });
    let model = SchematicModel::try_from(schematic_def)?;
    assert_eq!(model.get_name(), schematic_name);
    let result = SchematicValidator::validate_early_errors(&model);
    assert_eq!(result, Ok(()));
    let result = SchematicValidator::validate_final_errors(&model);
    assert_eq!(result, Ok(()));

    Ok(())
  }

  #[test_logger::test]
  fn test_missing_models() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.instances.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    let mut model = SchematicModel::try_from(schematic_def)?;
    model.allow_providers(&["test-namespace"]);
    let result = SchematicValidator::validate_final_errors(&model);
    assert_eq!(
      result,
      Err(ValidationError::new(
        schematic_name,
        vec![ValidationErrorKind::MissingComponentModel(
          "logger (test-namespace::log)".to_owned()
        )]
      ))
    );

    Ok(())
  }

  #[test_logger::test]
  fn test_missing_providers() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.instances.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    let model = SchematicModel::try_from(schematic_def)?;
    let result = SchematicValidator::validate_final_errors(&model);
    assert_eq!(
      result,
      Err(ValidationError::new(
        schematic_name,
        vec![ValidationErrorKind::MissingProvider(
          "test-namespace::log".to_owned(),
          "test-namespace".to_owned()
        )]
      ))
    );

    Ok(())
  }

  #[test_logger::test]
  fn test_finish_initialization() -> TestResult<()> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push("test-namespace".to_owned());
    schematic_def.instances.insert(
      "logger".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    schematic_def.connections.push(ConnectionDefinition::new(
      Target::new(SCHEMATIC_INPUT, "input"),
      Target::new("logger", "input"),
    ));
    schematic_def.connections.push(ConnectionDefinition::new(
      Target::new("logger", "output"),
      Target::new(SCHEMATIC_OUTPUT, "output"),
    ));
    let mut model = SchematicModel::try_from(schematic_def)?;
    let provider = ProviderModel {
      components: hashmap! {
        "log".to_owned() => ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![("input", "string")].try_into()?,
          outputs: vec![("output", "bytes")].try_into()?
        }.into()
      },
    };
    model.commit_providers(hashmap! {"test-namespace".to_owned()=> provider})?;
    let result = SchematicValidator::_validate(&model);
    assert_eq!(result, Ok(()));
    model.partial_initialization()?;
    let sig = model.get_signature().unwrap();
    assert_eq!(sig.inputs, vec![("input", "string")].try_into()?);
    assert_eq!(sig.outputs, vec![("output", "bytes")].try_into()?);

    Ok(())
  }
}
