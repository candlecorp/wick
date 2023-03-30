use std::path::{Path, PathBuf};
use std::sync::Arc;

use assets::{Asset, Progress, Status};
use bytes::{Bytes, BytesMut};
use futures::Stream;
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, trace};

use crate::Error;

#[derive(Debug, Clone)]
#[must_use]
pub struct LocationReference {
  pub(crate) location: String,
  pub(crate) baseurl: Arc<RwLock<Option<String>>>,
}

impl std::fmt::Display for LocationReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.location)
  }
}

#[derive(Debug, Default, Clone)]
#[must_use]
pub struct FetchOptions {
  pub(crate) allow_latest: bool,
  pub(crate) allow_insecure: Vec<String>,
}

impl FetchOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn allow_latest(mut self, allow_latest: bool) -> Self {
    self.allow_latest = allow_latest;
    self
  }

  pub fn allow_insecure(mut self, allow_insecure: impl AsRef<[String]>) -> Self {
    self.allow_insecure = allow_insecure.as_ref().to_owned();
    self
  }
}

impl PartialEq for LocationReference {
  fn eq(&self, other: &Self) -> bool {
    self.location == other.location && *self.baseurl.read() == *other.baseurl.read()
  }
}

impl LocationReference {
  /// Create a new location reference.
  pub fn new(location: impl AsRef<str>) -> Self {
    Self {
      location: location.as_ref().to_owned(),
      baseurl: Default::default(),
    }
  }

  pub fn baseurl(&self) -> PathBuf {
    self
      .baseurl
      .read()
      .as_ref()
      .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from)
  }

  pub fn path(&self) -> PathBuf {
    trace!(baseurl=?self.baseurl.read(), location=?self.location, "asset location");
    let mut path = self
      .baseurl
      .read()
      .as_ref()
      .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);
    path.push(&self.location);
    path
  }

  #[must_use]
  pub fn location(&self) -> &str {
    &self.location
  }

  pub async fn bytes(&self, options: &FetchOptions) -> Result<Bytes, Error> {
    let bytes = self
      .fetch(options.clone())
      .await
      .map_err(|err| Error::LoadError(self.path().to_string_lossy().into(), err.to_string()))?;
    Ok(bytes.into())
  }

  fn retrieve_as_file(&self) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let mut buffer = [0u8; 1024];

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let location = self.location.clone();
    let path = self.path();
    tokio::spawn(async move {
      let mut bm = BytesMut::new();
      let mut file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => {
          let _ = tx.send(Progress::new(&location, Status::Error(err.to_string())));
          return;
        }
      };

      let file_size = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(err) => {
          let _ = tx.send(Progress::new(&location, Status::Error(err.to_string())));
          return;
        }
      };

      let _ = tx.send(Progress::new(
        &location,
        Status::Progress {
          num: 0,
          total: file_size as _,
        },
      ));
      loop {
        let bytes = match file.read(&mut buffer).await {
          Ok(0) => break,
          Ok(bytes) => bytes,
          Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
          Err(e) => {
            let _ = tx.send(Progress::new(&location, Status::Error(e.to_string())));
            return;
          }
        };
        let _ = tx.send(Progress::new(
          &location,
          Status::Progress {
            num: bytes,
            total: file_size as _,
          },
        ));
        bm.extend_from_slice(&buffer[..bytes]);
      }

      let _ = tx.send(Progress::new(&location, Status::AssetComplete(bm.to_vec())));
    });
    Box::pin(UnboundedReceiverStream::new(rx))
  }

  fn retrieve_as_oci_with_progress(
    &self,
    options: FetchOptions,
  ) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let location = self.location.clone();
    tokio::spawn(async move {
      let _ = tx.send(Progress::new(&location, Status::Progress { num: 0, total: 0 }));
      match wick_oci_utils::fetch_oci_bytes(&location, options.allow_latest, &options.allow_insecure).await {
        Ok(bytes) => {
          let _ = tx.send(Progress::new(
            &location,
            Status::Progress {
              num: bytes.len(),
              total: bytes.len(),
            },
          ));
          let _ = tx.send(Progress::new(&location, Status::AssetComplete(bytes)));
        }
        Err(e) => {
          let _ = tx.send(Progress::new(&location, Status::Error(e.to_string())));
        }
      }
    });
    Box::pin(UnboundedReceiverStream::new(rx))
  }
}

impl Asset for LocationReference {
  type Options = FetchOptions;

  #[allow(clippy::expect_used)]
  fn set_baseurl(&self, baseurl: &str) {
    url::Url::parse(baseurl).map_or_else(
      |_| {
        let mut path = if baseurl.starts_with('.') {
          let mut path = std::env::current_dir().expect("failed to get current dir");
          path.push(baseurl);
          path
        } else {
          PathBuf::from(baseurl)
        };
        // Assume that a path with an extension is a file and the basedir
        // is the parent directory.
        if path.extension().is_some() {
          path.set_file_name("");
        }
        *self.baseurl.write() = Some(path.to_string_lossy().to_string());
      },
      |url| {
        *self.baseurl.write() = Some(url.to_string());
      },
    );
  }

  fn fetch_with_progress(&self, options: FetchOptions) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let path = self.path();
    debug!(path = ?path, "fetching asset with progress");
    if path.exists() {
      debug!(path = self.location, "load as file");
      self.retrieve_as_file()
    } else {
      debug!(url = self.location, "load as oci");
      self.retrieve_as_oci_with_progress(options)
    }
  }

  fn fetch(
    &self,
    options: FetchOptions,
  ) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<Vec<u8>, assets::Error>> + Send + Sync>> {
    let location = self.location.clone();
    let path = self.path();
    debug!(path = ?path, "fetching asset");
    Box::pin(async move {
      if Path::new(&path).exists() {
        let mut file = tokio::fs::File::open(&path)
          .await
          .map_err(|err| assets::Error::FileOpen(path.to_string_lossy().to_string(), err.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
      } else {
        let bytes = retrieve_as_oci(&location, options)
          .await
          .map_err(|err| assets::Error::FileOpen(path.to_string_lossy().to_string(), err.to_string()))?;
        Ok(bytes)
      }
    })
  }

  fn name(&self) -> &str {
    &self.location
  }
}

async fn retrieve_as_oci(location: &str, options: FetchOptions) -> Result<Vec<u8>, Error> {
  match wick_oci_utils::fetch_oci_bytes(location, options.allow_latest, &options.allow_insecure).await {
    Ok(bytes) => Ok(bytes),
    Err(e) => Err(Error::LoadError(location.to_owned(), e.to_string())),
  }
}

pub const CACHE_ROOT: &str = ".wick";

pub fn cache_location(bucket: &str, reference: &str) -> Result<PathBuf, Error> {
  let bucket = bucket.replace([':', '/', '.'], "_");
  let path = PathBuf::from(bucket);
  let path = path.join(CACHE_ROOT);
  let reference = reference.replace([':', '/', '.'], "_");
  let path = path.join(reference);
  std::fs::create_dir_all(&path).map_err(|error| Error::PackageCacheError {
    path: path.clone(),
    error,
  })?;

  Ok(path)
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_basic() -> Result<()> {
    let location = LocationReference::new("Cargo.toml");
    let mut expected = std::env::current_dir().unwrap();
    expected.push("Cargo.toml");
    assert_eq!(location.path(), expected);
    assert!(location.path().exists());
    location.set_baseurl("/etc");
    let mut expected = PathBuf::from("/etc");
    expected.push("Cargo.toml");
    assert_eq!(location.path(), expected);
    assert!(!location.path().exists());

    Ok(())
  }
}
