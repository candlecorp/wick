use url::Url;

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
  #[error("Bad configuration: {0}")]
  Validation(String),

  #[error("Failed to render path template {0}: {1}")]
  PathTemplate(String, String),

  #[error("Received invalid header value for header {0}")]
  InvalidHeader(String),

  #[error("Could not find operation {0} on this component")]
  OpNotFound(String),

  #[error("Invalid baseurl: {0}")]
  InvalidBaseUrl(Url),

  #[error("Proxy and baseurl must not be the same: {0}")]
  ProxyLoop(Url),

  #[error("Conversion error: {0}")]
  Conversion(String),
}
