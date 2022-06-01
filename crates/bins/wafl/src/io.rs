use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum IoError {
  #[error("Could not read '{0}': {1}")]
  Read(String, std::io::Error),
  #[error("Could not write '{0}': {1}")]
  Write(String, std::io::Error),
  #[error("Could not create directory '{0}': {1}")]
  CreateDirectory(String, std::io::Error),
}

pub(crate) async fn async_write(
  path: impl AsRef<Path> + Send + Sync,
  contents: impl AsRef<[u8]> + Send + Sync,
) -> Result<(), IoError> {
  tokio::fs::write(path.as_ref(), contents)
    .await
    .map_err(|e| IoError::Write(path_to_string(path), e))
}

pub(crate) async fn async_read(path: impl AsRef<Path> + Send + Sync) -> Result<Vec<u8>, IoError> {
  tokio::fs::read(path.as_ref())
    .await
    .map_err(|e| IoError::Read(path_to_string(path), e))
}

pub(crate) async fn async_read_to_string(path: impl AsRef<Path> + Send + Sync) -> Result<String, IoError> {
  tokio::fs::read_to_string(path.as_ref())
    .await
    .map_err(|e| IoError::Read(path_to_string(path), e))
}

pub(crate) async fn async_mkdirp(path: impl AsRef<Path> + Send + Sync) -> Result<(), IoError> {
  tokio::fs::create_dir_all(path.as_ref())
    .await
    .map_err(|e| IoError::CreateDirectory(path_to_string(path), e))
}

fn path_to_string(path: impl AsRef<Path> + Send + Sync) -> String {
  path.as_ref().to_string_lossy().to_string()
}
