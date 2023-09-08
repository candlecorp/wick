mod pull;
mod push;

use std::path::PathBuf;

pub use pull::*;
pub use push::*;
/// Annotation types associated with Wick packages.
pub mod annotations;
/// Media types associated with Wick packages.
pub mod media_types;

/// Represents a single file in a Wick package.
#[derive(Debug, Clone)]
pub struct PackageFile {
  path: PathBuf,
  hash: String,
  media_type: String,
  contents: bytes::Bytes,
}

impl PackageFile {
  pub fn new(path: PathBuf, hash: String, media_type: String, contents: bytes::Bytes) -> Self {
    Self {
      path,
      hash,
      media_type,
      contents,
    }
  }

  /// Get path for the file.
  #[must_use]
  pub const fn path(&self) -> &PathBuf {
    &self.path
  }

  /// Get hash for the file.
  #[must_use]
  pub fn hash(&self) -> &str {
    &self.hash
  }

  /// Get media type for the file.
  #[must_use]
  pub fn media_type(&self) -> &str {
    &self.media_type
  }

  /// Get contents for the file.
  #[must_use]
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }

  /// Get contents for the file.
  #[must_use]
  #[allow(clippy::missing_const_for_fn)]
  pub fn into_contents(self) -> bytes::Bytes {
    self.contents
  }
}
