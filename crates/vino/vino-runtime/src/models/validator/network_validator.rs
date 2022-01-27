use super::SchematicValidator;
use crate::dev::prelude::*;

pub(crate) struct NetworkValidator<'a> {
  model: &'a NetworkModel,
}

impl<'a> NetworkValidator<'a> {
  pub(crate) fn new(model: &'a NetworkModel) -> Self {
    NetworkValidator { model }
  }

  pub(crate) fn validate(model: &'a NetworkModel) -> Result<(), NetworkValidationError> {
    let validator = NetworkValidator::new(model);
    let name = model.get_name().cloned().unwrap_or_else(|| "Network".to_owned());

    let results: Vec<ValidationErrorKind> = vec![validator.validate_schematics()]
      .into_iter()
      .filter_map(std::result::Result::err)
      .flatten()
      .collect();

    if results.is_empty() {
      Ok(())
    } else {
      Err(NetworkValidationError::new(&name, results))
    }
  }

  fn validate_schematics(&self) -> Result<(), Vec<ValidationErrorKind>> {
    let mut errors = vec![];
    for schematic in self.model.get_schematics() {
      if let Err(e) = SchematicValidator::validate_final_errors(&schematic.read()) {
        errors.push(ValidationErrorKind::InvalidSchematic(InvalidSchematic::new(e)));
      };
    }
    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }
}

#[cfg(test)]
mod tests {

  use std::collections::HashMap;
  use std::convert::TryInto;

  use super::*;
  use crate::test::prelude::*;
  use crate::VINO_V0_NAMESPACE;
  #[test_logger::test]
  fn test_validate() -> TestResult<()> {
    let def = load_network_definition("./src/models/test-manifests/logger.yaml")?;
    let mut model = NetworkModel::try_from(def)?;
    let provider = ProviderModel {
      components: hashmap! {
        "log".to_owned() => ComponentSignature {
          name: "log".to_owned(),
          inputs: vec![("input", "string")].try_into()?,
          outputs: vec![("output", "string")].try_into()?,
        }.into()
      },
    };
    let mut providers = HashMap::new();
    providers.insert(VINO_V0_NAMESPACE.to_owned(), provider);
    model.update_providers(providers)?;
    NetworkValidator::validate(&model)?;

    Ok(())
  }

  #[test_logger::test]
  fn test_missing_provider() -> TestResult<()> {
    let def = load_network_definition("./manifests/v0/validation/missing-provider.yaml")?;
    let mut model = NetworkModel::try_from(def)?;
    let provider = ProviderModel {
      components: hashmap! {
        "validate".to_owned() => ComponentSignature {
          name: "validate".to_owned(),
          inputs: vec![("input", "string")].try_into()?,
          outputs: vec![("output", "string")].try_into()?
        }.into()
      },
    };
    let mut providers = HashMap::new();
    providers.insert("wapc".to_owned(), provider);
    model.update_providers(providers)?;
    let result = NetworkValidator::validate(&model);
    assert!(result.is_err());

    Ok(())
  }
}
