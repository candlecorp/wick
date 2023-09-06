use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
#[allow(missing_copy_implementations)]
#[non_exhaustive]
/// Crate error.
pub enum Error {
  /// Failed to get current working directory
  #[error("Failed to get current working directory")]
  Pwd,
  /// Settings has no source path and can't be saved.
  #[error("Settings has no source path and can not be saved")]
  NoSource,
  /// Could not save settings.
  #[error("Could not save settings at {}: {}", .0.display(), .1)]
  SaveFailed(PathBuf, std::io::Error),
}
