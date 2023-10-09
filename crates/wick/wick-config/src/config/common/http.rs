use std::fmt::Write;

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
  /// Raw Text Data
  Text = 3,
  /// Event Stream Data
  EventStream = 4,
}

impl Default for Codec {
  fn default() -> Self {
    Self::Json
  }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct HttpEvent {
  /// The event name if given
  event: Option<String>,
  /// The event data
  data: String,
  /// The event id if given
  id: Option<String>,
  /// Retry duration if given
  retry: Option<std::time::Duration>,
}

impl HttpEvent {
  #[must_use]
  pub const fn new(
    event: Option<String>,
    data: String,
    id: Option<String>,
    retry: Option<std::time::Duration>,
  ) -> Self {
    Self { event, data, id, retry }
  }

  #[must_use]
  pub const fn get_event(&self) -> &Option<String> {
    &self.event
  }
  #[must_use]
  pub const fn get_data(&self) -> &String {
    &self.data
  }
  #[must_use]
  pub const fn get_id(&self) -> &Option<String> {
    &self.id
  }
  #[must_use]
  pub const fn get_retry(&self) -> &Option<std::time::Duration> {
    &self.retry
  }

  #[must_use]
  pub fn to_sse_string(&self) -> String {
    let mut sse_string = String::new();

    if let Some(ref event) = self.event {
      writeln!(sse_string, "event: {}", event).unwrap();
    }

    // Splitting data by newline to ensure each line is prefixed with "data: "
    for line in self.data.split('\n') {
      writeln!(sse_string, "data: {}", line).unwrap();
    }

    if let Some(ref id) = self.id {
      writeln!(sse_string, "id: {}", id).unwrap();
    }

    if let Some(ref retry) = self.retry {
      // Converting retry duration to milliseconds
      let millis = retry.as_millis();
      writeln!(sse_string, "retry: {}", millis).unwrap();
    }

    // Adding the required empty line to separate events
    sse_string.push_str("\n");

    sse_string
  }
}

impl std::fmt::Display for Codec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Codec::Json => write!(f, "json"),
      Codec::Raw => write!(f, "raw"),
      Codec::FormData => write!(f, "form-data"),
      Codec::Text => write!(f, "text"),
      Codec::EventStream => write!(f, "event-stream"),
    }
  }
}
