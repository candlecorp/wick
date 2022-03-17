#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum PortStatus {
  // Ports that have already served a purpose fall into an idle state and can be closed.
  Idle,

  // A port that has data.
  Closed,
  // Ports that generated an error.

  // Invalid port (i.e. a "None" variant)
  Invalid,
}

impl std::fmt::Display for PortStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PortStatus::Idle => "Idle",

        PortStatus::Closed => "Closed",

        PortStatus::Invalid => "Invalid",
      }
    )
  }
}
