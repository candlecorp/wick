use crate::config;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
/// A component made out of other components
pub struct HttpClientComponentConfig {
  /// The URL base to use.
  #[asset(skip)]
  pub resource: String,

  /// The codec to use when encoding/decoding data.
  #[asset(skip)]
  pub codec: Option<Codec>,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  pub operations: Vec<HttpClientOperationDefinition>,
}

impl HttpClientComponentConfig {
  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    let codec = self.codec;
    self
      .operations
      .clone()
      .into_iter()
      .map(|mut op| {
        op.codec = op.codec.or(codec);
        op
      })
      .map(Into::into)
      .collect()
  }
}

impl From<HttpClientOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: HttpClientOperationDefinition) -> Self {
    Self {
      name: operation.name,
      inputs: operation.inputs,
      outputs: vec![
        // TODO: support actual HTTP Response type.
        wick_interface_types::Field::new("response", wick_interface_types::TypeSignature::Object),
        wick_interface_types::Field::new(
          "body",
          match operation.codec {
            Some(Codec::Json) => wick_interface_types::TypeSignature::Object,
            Some(Codec::Raw) => wick_interface_types::TypeSignature::Bytes,
            None => wick_interface_types::TypeSignature::Object,
          },
        ),
      ],
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct HttpClientOperationDefinition {
  /// The name of the operation.
  pub name: String,

  /// Types of the inputs to the operation.
  pub inputs: Vec<wick_interface_types::Field>,

  /// The path to append to our base URL, processed as a liquid template with each input as part of the template data.
  pub path: String,

  /// The codec to use when encoding/decoding data.
  pub codec: Option<Codec>,

  /// The HTTP method to use.
  pub method: HttpMethod,
}

impl HttpClientOperationDefinition {
  /// Create a new GET operation.
  pub fn new_get(name: impl AsRef<str>, path: impl AsRef<str>, inputs: Vec<wick_interface_types::Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      inputs,
      path: path.as_ref().to_owned(),
      method: HttpMethod::Get,
      codec: Default::default(),
    }
  }

  /// Create a new POST operation.
  pub fn new_post(name: impl AsRef<str>, path: impl AsRef<str>, inputs: Vec<wick_interface_types::Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      inputs,
      path: path.as_ref().to_owned(),
      method: HttpMethod::Post,
      codec: Default::default(),
    }
  }

  /// Create a new PUT operation.
  pub fn new_put(name: impl AsRef<str>, path: impl AsRef<str>, inputs: Vec<wick_interface_types::Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      inputs,
      path: path.as_ref().to_owned(),
      method: HttpMethod::Put,
      codec: Default::default(),
    }
  }

  /// Create a new DELETE operation.
  pub fn new_delete(name: impl AsRef<str>, path: impl AsRef<str>, inputs: Vec<wick_interface_types::Field>) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      inputs,
      path: path.as_ref().to_owned(),
      method: HttpMethod::Delete,
      codec: Default::default(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Supported HTTP methods
pub enum HttpMethod {
  Get = 0,
  Post = 1,
  Put = 2,
  Delete = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Codec to use when encoding/decoding data.
pub enum Codec {
  /// JSON Codec
  Json = 0,
  /// Raw
  Raw = 1,
}

impl Default for Codec {
  fn default() -> Self {
    Self::Json
  }
}
