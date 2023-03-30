/// Errors that can occur when processing assets.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// Asset not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not read asset at the specified location.
  #[error("Error opening file {0}: {1}")]
  FileOpen(String, String),

  /// Could not load file.
  #[error("Could not read file {0}")]
  LoadError(String),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Self::LoadError(e.to_string())
  }
}
