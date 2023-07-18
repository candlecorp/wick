use tracing::warn;
use wick_interface_types::{Field, Type};

use crate::{Error, RuntimeConfig};

pub fn expect_configuration_matches(name: &str, config: Option<&RuntimeConfig>, fields: &[Field]) -> Result<(), Error> {
  let fields = fields
    .iter()
    .filter(|f| f.required() || !matches!(f.ty(), Type::Optional { .. }))
    .collect::<Vec<&Field>>();

  if config.is_none() {
    if fields.is_empty() {
      return Ok(());
    }
    warn!(?config, ?fields, "configuration invalid");
    return Err(Error::Signature(
      name.to_owned(),
      format!(
        "missing field(s): {}",
        fields
          .iter()
          .map(|f| f.name().to_owned())
          .collect::<Vec<String>>()
          .join(", ")
      ),
    ));
  }

  let config = config.unwrap();

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
  #[case(json!({"required_field": "value"}),vec![Field::new("required_field", Type::String)])]
  #[case(json!({"optional_field": "value"}),vec![Field::new("optional_field", Type::Optional { ty: Box::new(Type::String)})])]
  #[case(json!({"optional_field": serde_json::Value::Null}),vec![Field::new("optional_field", Type::Optional { ty: Box::new(Type::String)})])]
  #[case(json!({}),vec![Field::new("optional_field", Type::Optional { ty: Box::new(Type::String)})])]
  #[case(json!({}),vec![])]

  fn config_validation_positive(#[case] config: serde_json::Value, #[case] fields: Vec<Field>) -> Result<()> {
    let config = Some(RuntimeConfig::try_from(config)?);
    expect_configuration_matches("test", config.as_ref(), &fields)?;

    Ok(())
  }

  #[rstest::rstest]
  #[case(json!({}),vec![Field::new("required_field", Type::String)])]
  fn config_validation_negative(#[case] config: serde_json::Value, #[case] fields: Vec<Field>) -> Result<()> {
    let config = Some(RuntimeConfig::try_from(config)?);
    assert!(expect_configuration_matches("test", config.as_ref(), &fields).is_err());

    Ok(())
  }
}
