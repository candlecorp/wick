#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
/// Supported HTTP methods
#[serde(rename_all = "kebab-case")]
pub enum HttpMethod {
  Get = 0,
  Post = 1,
  Put = 2,
  Delete = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
/// Codec to use when encoding/decoding data.
#[serde(rename_all = "kebab-case")]
pub enum Codec {
  /// JSON Codec
  Json = 0,
  /// Raw
  Raw = 1,
  /// Form Data
  FormData = 2,
  /// Xml Data
  Xml = 3,
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
      Codec::Xml => write!(f, "xml"),
    }
  }
}
