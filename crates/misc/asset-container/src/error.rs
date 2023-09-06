use std::path::PathBuf;

/// Errors that can occur when processing assets.
#[derive(thiserror::Error, Clone, Debug)]
#[non_exhaustive]
pub enum Error {
  /// Asset not found at the specified path.
  #[error("File not found {0}")]
  FileNotFound(String),

  /// Could not read asset at the specified location.
  #[error("Error opening file {}: {1}", .0.display())]
  FileOpen(PathBuf, String),

  /// Could not fetch remote asset.
  #[error("Error fetching file {0}: {1}")]
  RemoteFetch(String, String),

  /// Could not load file.
  #[error("Could not read file {0}")]
  LoadError(String),

  /// Could not fetch directory as bytes.
  #[error("Can not fetch directory bytes {}", .0.display())]
  IsDirectory(PathBuf),

  /// The location of the asset was in a format the Asset couldn't parse.
  #[error("Could not parse location format: {0}")]
  Parse(String),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Self::LoadError(e.to_string())
  }
}
