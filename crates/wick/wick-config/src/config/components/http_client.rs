#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;

use crate::config;

#[derive(Debug, Clone, Builder, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
#[must_use]
/// A component made out of other components
pub struct HttpClientComponentConfig {
  /// The URL base to use.
  #[asset(skip)]
  pub(crate) resource: String,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Vec<wick_interface_types::Field>,

  /// The codec to use when encoding/decoding data.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) codec: Option<Codec>,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) operations: Vec<HttpClientOperationDefinition>,
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
      config: operation.config,
      inputs: operation.inputs,
      outputs: vec![
        // TODO: support actual HTTP Response type.
        wick_interface_types::Field::new("response", wick_interface_types::Type::Object),
        wick_interface_types::Field::new(
          "body",
          match operation.codec {
            Some(Codec::Json) => wick_interface_types::Type::Object,
            Some(Codec::Raw) => wick_interface_types::Type::Bytes,
            Some(Codec::FormData) => wick_interface_types::Type::Object,
            None => wick_interface_types::Type::Object,
          },
        ),
      ],
    }
  }
}

#[derive(Debug, Clone, Builder, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[builder(setter(into))]
#[must_use]
/// An operation whose implementation is an HTTP request.
pub struct HttpClientOperationDefinition {
  /// The name of the operation.
  pub(crate) name: String,

  /// The configuration the operation needs.
  #[builder(default)]
  pub(crate) config: Vec<wick_interface_types::Field>,

  /// Types of the inputs to the operation.
  pub(crate) inputs: Vec<wick_interface_types::Field>,

  /// The path to append to our base URL, processed as a liquid template with each input as part of the template data.
  pub(crate) path: String,

  /// The codec to use when encoding/decoding data.
  #[builder(default)]
  pub(crate) codec: Option<Codec>,

  /// The body to send with the request.
  #[builder(default)]
  pub(crate) body: Option<liquid_json::LiquidJsonValue>,

  /// The headers to send with the request.
  #[builder(default)]
  pub(crate) headers: Option<HashMap<String, Vec<String>>>,

  /// The HTTP method to use.
  pub(crate) method: HttpMethod,
}

impl HttpClientOperationDefinition {
  /// Create a new GET operation.
  pub fn new_get(
    name: impl AsRef<str>,
    path: impl AsRef<str>,
    inputs: Vec<wick_interface_types::Field>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name.as_ref())
      .inputs(inputs)
      .path(path.as_ref())
      .headers(headers)
      .method(HttpMethod::Get);
    builder
  }

  /// Create a new POST operation.
  pub fn new_post(
    name: impl AsRef<str>,
    path: impl AsRef<str>,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name.as_ref())
      .inputs(inputs)
      .path(path.as_ref())
      .body(body)
      .headers(headers)
      .method(HttpMethod::Post);
    builder
  }

  /// Create a new PUT operation.
  pub fn new_put(
    name: impl AsRef<str>,
    path: impl AsRef<str>,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name.as_ref())
      .inputs(inputs)
      .path(path.as_ref())
      .body(body)
      .headers(headers)
      .method(HttpMethod::Put);
    builder
  }

  /// Create a new DELETE operation.
  pub fn new_delete(
    name: impl AsRef<str>,
    path: impl AsRef<str>,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name.as_ref())
      .inputs(inputs)
      .path(path.as_ref())
      .body(body)
      .headers(headers)
      .method(HttpMethod::Delete);
    builder
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Supported HTTP methods
pub enum HttpMethod {
  /// HTTP GET method
  Get = 0,
  /// HTTP POST method
  Post = 1,
  /// HTTP PUT method
  Put = 2,
  /// HTTP DELETE method
  Delete = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Codec to use when encoding/decoding data.
pub enum Codec {
  /// JSON Codec
  Json = 0,
  /// Raw
  Raw = 1,
  /// Form Data
  FormData = 2,
}

impl Default for Codec {
  fn default() -> Self {
    Self::Json
  }
}

impl std::fmt::Display for Codec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Codec::Json => write!(f, "json"),
      Codec::Raw => write!(f, "raw"),
      Codec::FormData => write!(f, "form-data"),
    }
  }
}
