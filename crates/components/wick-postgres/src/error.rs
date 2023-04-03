#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Temp")]
  Temp,
  #[error("Invalid float")]
  InvalidFloat,
  #[error("Cannot convert pg-cell \"{0}\" of type \"{1}\" to a JSONValue.")]
  Conversion(String, String),

  #[error("Could not retrieve simple data for column {0}: {1}")]
  GetBasic(String, tokio_postgres::Error),

  #[error("Could not retrieve array data for column {0}: {1}")]
  GetArray(String, tokio_postgres::Error),
}
