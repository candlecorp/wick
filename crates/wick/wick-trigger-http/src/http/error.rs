use wick_config::config::Codec;
use wick_packet::Entity;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::exhaustive_enums)]
pub enum HttpError {
  #[error("Internal error: {:?}",.0)]
  InternalError(InternalError),

  #[error("Operation error: {0}")]
  OperationError(String),

  #[error("Error in stream for '{0}': {1}")]
  OutputStream(String, String),

  #[error("Unsupported HTTP method: {0}")]
  UnsupportedMethod(String),

  #[error("Unsupported HTTP version: {0}")]
  UnsupportedVersion(String),

  #[error("Missing query parameters: {}", .0.join(", "))]
  MissingQueryParameters(Vec<String>),

  #[error("Could not decode body as JSON: {0}")]
  InvalidBody(serde_json::Error),

  #[error("Invalid status code: {0}")]
  InvalidStatusCode(String),

  #[error("Invalid parameter value: {0}")]
  InvalidParameter(String),

  #[error("Could not serialize output into '{0}' codec: {1}")]
  Codec(Codec, String),

  #[error("Could not decode stream item as base64 bytes: {0}")]
  Bytes(String),

  #[error("Could not decode stream item as a utf-8 string: {0}")]
  Utf8Text(String),

  #[error("Could not decode stream item as HttpEvent: {0}")]
  HttpEvent(String),

  #[error("Invalid header name: {0}")]
  InvalidHeaderName(String),

  #[error("Invalid header value: {0}")]
  InvalidHeaderValue(String),

  #[error("Invalid path or query parameters: {0}")]
  InvalidUri(String),

  #[error("Invalid pre-request middleware response: {0}")]
  InvalidPreRequestResponse(String),

  #[error("Pre-request middleware '{0}' did not provide a request or response")]
  PreRequestResponseNoData(Entity),

  #[error("Post-request middleware '{0}' did not provide a response")]
  PostRequestResponseNoData(Entity),

  #[error("Invalid post-request middleware response: {0}")]
  InvalidPostRequestResponse(String),

  #[error("Error deserializing response on port {0}: {1}")]
  Deserialize(String, String),

  #[error("URI {0} could not be parsed: {1}")]
  RouteSyntax(String, String),

  #[error("{0}")]
  InitializationFailed(String),

  #[error("error in configuration: {0}")]
  Config(Box<wick_config::Error>),
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum InternalError {
  Builder,
}

impl From<HttpError> for wick_trigger::Error {
  fn from(value: HttpError) -> Self {
    wick_trigger::Error::new_context("http", wick_trigger::error::ErrorKind::Trigger(Box::new(value)))
  }
}

impl From<wick_config::Error> for HttpError {
  fn from(value: wick_config::Error) -> Self {
    HttpError::Config(Box::new(value))
  }
}
