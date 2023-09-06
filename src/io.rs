use std::path::{Path, PathBuf};

use serde_json::json;
use structured_output::StructuredOutput;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum IoError {
  #[error("{0} would be over-written by this operation. Use --force to over-write.")]
  WouldOverwrite(String),
  #[error("Could not read '{0}': {1}")]
  Read(String, std::io::Error),
  #[error("Could not write '{0}': {1}")]
  Write(String, std::io::Error),
  #[error("Could not create directory '{0}': {1}")]
  CreateDirectory(String, std::io::Error),
}

/*
 * These functions exist to generate more helpful error messages.
 * Do not refactor them out as needless abstraction unless
 * you address the error message concerns. -jo
 * */

pub(crate) async fn write_bytes(
  path: impl AsRef<Path> + Send + Sync,
  contents: impl AsRef<[u8]> + Send + Sync,
) -> Result<(), IoError> {
  tokio::fs::write(path.as_ref(), contents)
    .await
    .map_err(|e| IoError::Write(path_to_string(path), e))
}

#[allow(unused)]
pub(crate) async fn read_bytes(path: impl AsRef<Path> + Send + Sync) -> Result<Vec<u8>, IoError> {
  tokio::fs::read(path.as_ref())
    .await
    .map_err(|e| IoError::Read(path_to_string(path), e))
}

pub(crate) async fn read_to_string(path: impl AsRef<Path> + Send + Sync) -> Result<String, IoError> {
  tokio::fs::read_to_string(path.as_ref())
    .await
    .map_err(|e| IoError::Read(path_to_string(path), e))
}

pub(crate) async fn mkdirp(path: impl AsRef<Path> + Send + Sync) -> Result<(), IoError> {
  tokio::fs::create_dir_all(path.as_ref())
    .await
    .map_err(|e| IoError::CreateDirectory(path_to_string(path), e))
}

fn path_to_string(path: impl AsRef<Path> + Send + Sync) -> String {
  path.as_ref().to_string_lossy().to_string()
}

pub(crate) struct File {
  pub(crate) path: PathBuf,
  contents: Vec<u8>,
}

impl File {
  pub(crate) fn new(path: impl AsRef<Path>, contents: Vec<u8>) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
      contents,
    }
  }
}

pub(crate) async fn init_files(files: &[File], dry_run: bool) -> Result<StructuredOutput, IoError> {
  for file in files {
    if file.path.exists() {
      return Err(IoError::WouldOverwrite(path_to_string(&file.path)));
    }
  }

  for file in files {
    info!(file = %file.path.display(), "writing file");

    if dry_run {
      info!(
        "dry run: not writing {} bytes to {}",
        file.contents.len(),
        file.path.display()
      );
    } else {
      write_bytes(&file.path, &file.contents).await?;
    }
  }

  let as_string = format!(
    "Wrote: {}",
    files
      .iter()
      .map(|f| f.path.display().to_string())
      .collect::<Vec<_>>()
      .join(", ")
  );
  let as_json = json!({
    "files": files.iter().map(|f| f.path.display().to_string()).collect::<Vec<_>>(),
  });

  Ok(StructuredOutput::new(as_string, as_json))
}
