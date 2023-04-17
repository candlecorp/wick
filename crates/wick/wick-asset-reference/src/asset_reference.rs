use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use assets::{Asset, AssetManager, Progress, Status};
use bytes::{Bytes, BytesMut};
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::Stream;
use tracing::{debug, trace};

use crate::{normalize_path_str, Error};

#[derive(Debug, Clone)]
#[must_use]
pub struct AssetReference {
  pub(crate) location: String,
  pub(crate) cache_location: Arc<RwLock<Option<PathBuf>>>,
  pub(crate) baseurl: Arc<RwLock<Option<String>>>,
}

impl std::fmt::Display for AssetReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.location)
  }
}

#[derive(Debug, Default, Clone)]
#[must_use]
pub struct FetchOptions {
  pub(crate) allow_latest: bool,
  pub(crate) allow_insecure: Vec<String>,
  pub(crate) artifact_dir: Option<PathBuf>,
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

  pub fn artifact_dir(mut self, dir: PathBuf) -> Self {
    self.artifact_dir = Some(dir);
    self
  }

  #[must_use]
  pub fn get_artifact_dir(&self) -> Option<&PathBuf> {
    self.artifact_dir.as_ref()
  }

  #[must_use]
  pub fn get_allow_latest(&self) -> bool {
    self.allow_latest
  }

  #[must_use]
  pub fn get_allow_insecure(&self) -> &[String] {
    &self.allow_insecure
  }
}

impl PartialEq for AssetReference {
  fn eq(&self, other: &Self) -> bool {
    self.location == other.location && *self.baseurl.read() == *other.baseurl.read()
  }
}

impl AssetReference {
  /// Create a new location reference.
  pub fn new(location: impl AsRef<str>) -> Self {
    Self {
      location: location.as_ref().to_owned(),
      cache_location: Default::default(),
      baseurl: Default::default(),
    }
  }

  #[must_use]
  pub fn baseurl(&self) -> Option<String> {
    self.baseurl.read().clone()
  }

  pub fn path(&self) -> Result<String, Error> {
    trace!(baseurl=?self.baseurl.read(), location=?self.location, "asset location");
    if let Some(cache_loc) = self.cache_location.read().as_ref() {
      Ok(cache_loc.to_string_lossy().to_string())
    } else if self.location.starts_with('@') {
      Ok(self.location.trim_start_matches('@').to_owned())
    } else {
      let url = normalize_path_str(&self.location, self.baseurl())?;
      Ok(url)
    }
  }

  #[must_use]
  pub fn location(&self) -> &str {
    &self.location
  }

  pub async fn bytes(&self, options: &FetchOptions) -> Result<Bytes, Error> {
    match self.fetch(options.clone()).await {
      Ok(bytes) => Ok(bytes.into()),
      Err(err) => Err(Error::LoadError(self.path()?, err.to_string())),
    }
  }

  fn retrieve_as_file(&self) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let mut buffer = [0u8; 1024];

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let location = self.location.clone();
    tokio::spawn(async move {
      let location = location.as_str();
      let mut bm = BytesMut::new();
      let mut file = match tokio::fs::File::open(location).await {
        Ok(file) => file,
        Err(err) => {
          let _ = tx.send(Progress::new(location, Status::Error(err.to_string())));
          return;
        }
      };

      let file_size = match file.metadata().await {
        Ok(metadata) => metadata.len(),
        Err(err) => {
          let _ = tx.send(Progress::new(location, Status::Error(err.to_string())));
          return;
        }
      };

      let _ = tx.send(Progress::new(
        location,
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
            let _ = tx.send(Progress::new(location, Status::Error(e.to_string())));
            return;
          }
        };
        let _ = tx.send(Progress::new(
          location,
          Status::Progress {
            num: bytes,
            total: file_size as _,
          },
        ));
        bm.extend_from_slice(&buffer[..bytes]);
      }

      let _ = tx.send(Progress::new(location, Status::AssetComplete(bm.to_vec())));
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
      let location = location.as_str();
      let _ = tx.send(Progress::new(location, Status::Progress { num: 0, total: 0 }));
      match wick_oci_utils::fetch_oci_bytes(location, options.allow_latest, &options.allow_insecure).await {
        Ok(bytes) => {
          let _ = tx.send(Progress::new(
            location,
            Status::Progress {
              num: bytes.len(),
              total: bytes.len(),
            },
          ));
          let _ = tx.send(Progress::new(location, Status::AssetComplete(bytes)));
        }
        Err(e) => {
          let _ = tx.send(Progress::new(location, Status::Error(e.to_string())));
        }
      }
    });
    Box::pin(UnboundedReceiverStream::new(rx))
  }
}

impl Asset for AssetReference {
  type Options = FetchOptions;

  #[allow(clippy::expect_used)]
  fn update_baseurl(&self, baseurl: &str) {
    let baseurl = if baseurl.starts_with('.') {
      let mut path = std::env::current_dir().expect("failed to get current dir");
      path.push(baseurl);

      path.to_string_lossy().to_string()
    } else {
      baseurl.to_owned()
    };

    *self.baseurl.write() = Some(baseurl);
  }

  fn fetch_with_progress(&self, options: FetchOptions) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let path = self.path();

    debug!(path = ?path, "fetching asset with progress");
    match path {
      Ok(path) => {
        let path = PathBuf::from(path);
        if path.exists() {
          debug!(path = %path.display(), "load as file");
          self.retrieve_as_file()
        } else {
          debug!(url = %path.display(), "load as oci");
          self.retrieve_as_oci_with_progress(options)
        }
      }

      Err(e) => Box::pin(tokio_stream::once(Progress::new(
        self.location(),
        Status::Error(e.to_string()),
      ))),
    }
  }

  fn fetch(
    &self,
    options: FetchOptions,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<u8>, assets::Error>> + Send + Sync>> {
    let path = self.path();

    debug!(path = ?path, "fetching asset");
    let cache_location = self.cache_location.clone();
    Box::pin(async move {
      let path = path.map_err(|e| assets::Error::Parse(e.to_string()))?;
      let pb = PathBuf::from(&path);
      if pb.exists() {
        let mut file = tokio::fs::File::open(&path)
          .await
          .map_err(|err| assets::Error::FileOpen(path.clone(), err.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
      } else {
        let (cache_loc, bytes) = retrieve_remote(&path, options)
          .await
          .map_err(|err| assets::Error::FileOpen(path.clone(), err.to_string()))?;
        *cache_location.write() = Some(cache_loc);
        Ok(bytes)
      }
    })
  }

  fn name(&self) -> &str {
    self.location.as_str()
  }
}

async fn retrieve_remote(location: &str, options: FetchOptions) -> Result<(PathBuf, Vec<u8>), Error> {
  let oci_opts = wick_oci_utils::OciOptions::default()
    .cache_dir(options.get_artifact_dir().cloned())
    .allow_insecure(options.allow_insecure)
    .allow_latest(options.allow_latest);
  let result = wick_oci_utils::package::pull(location, &oci_opts)
    .await
    .map_err(|e| Error::LoadError(location.to_owned(), e.to_string()))?;
  let cache_location = result.base_dir.join(result.root_path);
  let bytes = tokio::fs::read(&cache_location)
    .await
    .map_err(|e| Error::LoadError(cache_location.display().to_string(), e.to_string()))?;
  Ok((cache_location, bytes))
}

impl AssetManager for AssetReference {
  type Asset = AssetReference;

  fn assets(&self) -> assets::Assets<Self::Asset> {
    assets::Assets::new(vec![self])
  }
}

impl TryFrom<String> for AssetReference {
  type Error = Error;
  fn try_from(val: String) -> Result<Self, Error> {
    Ok(Self::new(val))
  }
}

#[cfg(test)]
mod test {

  use std::path::PathBuf;

  use anyhow::Result;

  use super::*;

  #[test_logger::test]
  fn test_no_baseurl() -> Result<()> {
    let location = AssetReference::new("Cargo.toml");
    println!("location: {:?}", location);
    let mut expected = std::env::current_dir().unwrap();
    expected.push("Cargo.toml");
    let expected = expected.to_string_lossy();
    assert_eq!(location.path()?, expected.to_string());
    assert!(PathBuf::from(location.path()?).exists());
    Ok(())
  }

  #[test_logger::test]
  fn test_baseurl() -> Result<()> {
    let location = AssetReference::new("Cargo.toml");
    let mut root_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    root_project_dir.pop();
    root_project_dir.pop();
    root_project_dir.pop();

    location.set_baseurl(&root_project_dir.to_string_lossy());
    let mut expected = root_project_dir;
    expected.push("Cargo.toml");
    let expected = expected.to_string_lossy();
    assert_eq!(location.path()?, expected.to_string());
    assert!(PathBuf::from(location.path()?).exists());

    Ok(())
  }

  #[test_logger::test]
  fn test_relative_with_baseurl() -> Result<()> {
    let location = AssetReference::new("../Cargo.toml");
    let mut root_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    root_project_dir.pop();
    root_project_dir.pop();

    location.set_baseurl(&root_project_dir.to_string_lossy());
    let mut expected = root_project_dir;
    expected.pop();
    expected.push("Cargo.toml");
    let expected = expected.to_string_lossy();
    assert_eq!(location.path()?, expected.to_string());

    Ok(())
  }

  #[test_logger::test]
  fn test_relative_with_baseurl2() -> Result<()> {
    let location = AssetReference::new("../src/utils.rs");
    let mut crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    crate_dir.push("src");
    location.set_baseurl(&crate_dir.to_string_lossy());
    println!("crate_dir: {}", crate_dir.to_string_lossy());
    println!("actual: {:#?}", location);
    let mut expected = PathBuf::from(&crate_dir);
    expected.push("../src/utils.rs");
    println!("expected: {}", expected.to_string_lossy());

    let expected = expected.canonicalize()?;
    assert_eq!(location.path()?, expected.to_string_lossy().to_string());

    Ok(())
  }
}
