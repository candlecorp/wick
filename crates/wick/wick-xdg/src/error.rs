#[derive(thiserror::Error, Debug, Clone)]
#[allow(missing_copy_implementations)]
/// Crate error.
pub enum Error {
  /// Failed to get current working directory
  #[error("Failed to get current working directory")]
  Pwd,
}
