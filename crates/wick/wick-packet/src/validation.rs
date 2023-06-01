use wick_interface_types::Field;

use crate::{Error, GenericConfig};

pub fn expect_configuration_matches(
  name: &str,
  config: Option<&GenericConfig>,
  fields: Option<&[Field]>,
) -> Result<(), Error> {
  if config.is_none() {
    if fields.map_or_else(|| true, |f| f.is_empty()) {
      return Ok(());
    }
    return Err(Error::Signature(
      name.to_owned(),
      format!(
        "missing field(s): {}",
        fields
          .unwrap()
          .iter()
          .map(|f| f.name().to_owned())
          .collect::<Vec<String>>()
          .join(", ")
      ),
    ));
  }

  let config = config.unwrap();
  let fields = fields.unwrap_or_default();
  for field in fields {
    if !config.has(field.name()) {
      return Err(Error::Signature(
        name.to_owned(),
        format!("missing field: {}", field.name()),
      ));
    }
    // TODO: validate type.
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;
  use wick_interface_types::Type;

  use super::*;

  #[rstest::rstest]
  #[case(json!({"required_field": "value"}),Some(vec![Field::new("required_field", Type::String)]))]
  #[case(json!({}),Some(vec![]))]
  #[case(json!({}),None)]
  fn config_validation_positive(#[case] config: serde_json::Value, #[case] fields: Option<Vec<Field>>) -> Result<()> {
    let config = Some(GenericConfig::try_from(config)?);
    expect_configuration_matches("test", config.as_ref(), fields.as_deref())?;

    Ok(())
  }

  #[rstest::rstest]
  #[case(json!({}),Some(vec![Field::new("required_field", Type::String)]))]
  fn config_validation_negative(#[case] config: serde_json::Value, #[case] fields: Option<Vec<Field>>) -> Result<()> {
    let config = Some(GenericConfig::try_from(config)?);
    assert!(expect_configuration_matches("test", config.as_ref(), fields.as_deref()).is_err());

    Ok(())
  }
}
