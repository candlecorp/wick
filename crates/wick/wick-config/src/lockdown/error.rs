use crate::config::resources::ResourceKind;

/// Error returned when asserting a lockdown configuration fails.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LockdownError {
  failures: Vec<FailureKind>,
}

impl LockdownError {
  /// Instantiate a new lockdown error.
  #[must_use]
  pub fn new(failures: Vec<FailureKind>) -> Self {
    Self { failures }
  }

  /// Get the failures that occurred.
  #[must_use]
  pub fn failures(&self) -> &[FailureKind] {
    &self.failures
  }
}

impl std::error::Error for LockdownError {}
impl std::fmt::Display for LockdownError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "lockdown configuration resulted in {} failures: {}",
      self.failures.len(),
      self
        .failures
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}

/// Errors that occur when a configuration is invalid for a configuration.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FailureKind {
  /// General Error.
  General(String),
  /// A lockdown assertion failed.
  Failed(Box<LockdownError>),
  /// A component was not expressly allowed via this configuration.
  NotExpresslyAllowed(String, ResourceKind),
  /// A component is not allowed to access given volume.
  Volume(String, String),
  /// A volume could not be reconciled on the file system.
  VolumeInvalid(String, String),
  /// A restriction is not a valid path.
  VolumeRestrictionInvalid(String),
  /// A component is not allowed to access given port.
  Port(String, u16),
  /// A component is not allowed to access given address.
  Address(String, String),
  /// A component is not allowed to access given url.
  Url(String, String),
}

impl std::fmt::Display for FailureKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FailureKind::Failed(error) => write!(f, "{error}"),
      FailureKind::General(s) => f.write_str(s),
      FailureKind::VolumeInvalid(name, path) => write!(
        f,
        "the path {} for volume {} could not be reconciled on the file system",
        path, name
      ),
      FailureKind::Volume(id, path) => write!(f, "component {} is not allowed to access {}", id, path),
      FailureKind::NotExpresslyAllowed(id,kind) => write!(f, "component {} is not expressly allowed to access a {} resource", id,kind),
      FailureKind::VolumeRestrictionInvalid(path) => write!(
        f,
        "restricted volume '{}' is not valid, it can not be reconciled on the file system and can not be asserted against",
        path
      ),
        FailureKind::Port(id, port) =>  write!(f, "component {} is not allowed to access {}", id, port),
        FailureKind::Address(id,address) =>  write!(f, "component {} is not allowed to access {}", id, address),
        FailureKind::Url(id, url) =>  write!(f, "component {} is not allowed to access {}", id, url),
    }
  }
}
