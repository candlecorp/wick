use std::sync::Arc;

use assets::{Asset, AssetManager, Progress, Status};
use bytes::{Bytes, BytesMut};
use futures::Stream;
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, trace};
use url::Url;

use crate::error::ManifestError;
use crate::{str_to_url, Error};

#[derive(Debug, Clone)]
#[must_use]
pub struct LocationReference {
  pub(crate) location: String,
  pub(crate) baseurl: Arc<RwLock<Option<Url>>>,
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

  #[must_use]
  pub fn baseurl(&self) -> Url {
    self
      .baseurl
      .read()
      .clone()
      .unwrap_or_else(|| Url::from_file_path(std::env::current_dir().unwrap()).unwrap())
  }

  pub fn path(&self) -> Result<Url, ManifestError> {
    trace!(baseurl=?self.baseurl.read(), location=?self.location, "asset location");
    let url = str_to_url(&self.location, Some(self.baseurl()))?;
    Ok(url)
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

impl Asset for LocationReference {
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

    let url = match Url::parse(&baseurl) {
      Ok(url) => url,
      Err(_e) => match Url::from_file_path(&baseurl) {
        Ok(url) => url,
        Err(_e) => panic!("failed to parse baseurl: {}", baseurl),
      },
    };

    *self.baseurl.write() = Some(url);
  }

  fn fetch_with_progress(&self, options: FetchOptions) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    let path = self.path();

    debug!(path = ?path, "fetching asset with progress");
    match self.path() {
      Ok(path) => {
        if path.scheme() == "file" {
          debug!(path = %path, "load as file");
          self.retrieve_as_file()
        } else {
          debug!(url = %path, "load as oci");
          self.retrieve_as_oci_with_progress(options)
        }
      }
      Err(e) => Box::pin(futures::stream::once(async move {
        Progress::new(self.location(), Status::Error(e.to_string()))
      })),
    }
  }

  fn fetch(
    &self,
    options: FetchOptions,
  ) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<Vec<u8>, assets::Error>> + Send + Sync>> {
    let path = self.path();
    debug!(path = ?path, "fetching asset");
    Box::pin(async move {
      let path = path.map_err(|e| assets::Error::Parse(e.to_string()))?;
      if path.scheme() == "file" {
        let mut file = tokio::fs::File::open(
          path
            .to_file_path()
            .map_err(|_| assets::Error::FileOpen(path.to_string(), "Invalid URL".to_owned()))?,
        )
        .await
        .map_err(|err| assets::Error::FileOpen(path.to_string(), err.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
      } else {
        let bytes = retrieve_as_oci(&path, options)
          .await
          .map_err(|err| assets::Error::FileOpen(path.to_string(), err.to_string()))?;
        Ok(bytes)
      }
    })
  }

  fn name(&self) -> &str {
    self.location.as_str()
  }
}

async fn retrieve_as_oci(location: &Url, options: FetchOptions) -> Result<Vec<u8>, Error> {
  match wick_oci_utils::fetch_oci_bytes(location.as_str(), options.allow_latest, &options.allow_insecure).await {
    Ok(bytes) => Ok(bytes),
    Err(e) => Err(Error::LoadError(location.clone(), e.to_string())),
  }
}

impl AssetManager for LocationReference {
  type Asset = LocationReference;

  fn assets(&self) -> assets::Assets<Self::Asset> {
    assets::Assets::new(vec![self])
  }
}

#[cfg(test)]
mod test {

  use std::path::PathBuf;

  use anyhow::Result;

  use super::*;

  #[test]
  fn test_no_baseurl() -> Result<()> {
    let location = LocationReference::new("Cargo.toml");
    let mut expected = std::env::current_dir().unwrap();
    expected.push("Cargo.toml");
    let expected = Url::from_file_path(expected).unwrap();
    assert_eq!(location.path()?.to_string(), expected.to_string());
    assert!(location.path()?.to_file_path().unwrap().exists());
    Ok(())
  }

  #[test]
  fn test_baseurl() -> Result<()> {
    let location = LocationReference::new("Cargo.toml");
    location.set_baseurl("/etc");
    let mut expected = PathBuf::from("/etc");
    expected.push("Cargo.toml");
    let expected = Url::from_file_path(expected).unwrap();
    assert_eq!(location.path()?.to_string(), expected.to_string());
    assert!(!location.path()?.to_file_path().unwrap().exists());

    Ok(())
  }

  #[test]
  fn test_relative_with_baseurl() -> Result<()> {
    let location = LocationReference::new("../Cargo.toml");
    location.set_baseurl("/this/that/other");
    let mut expected = PathBuf::from("/this/that/other");
    expected.pop();
    expected.push("Cargo.toml");
    let expected = Url::from_file_path(expected).unwrap();
    assert_eq!(location.path()?.to_string(), expected.to_string());

    Ok(())
  }

  #[test]
  fn test_relative_with_baseurl2() -> Result<()> {
    let location = LocationReference::new("../src/utils.rs");
    let mut crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    crate_dir.push("src");
    location.set_baseurl(&crate_dir.to_string_lossy());
    println!("crate_dir: {}", crate_dir.to_string_lossy());
    println!("actual: {:#?}", location);
    let mut expected = PathBuf::from(&crate_dir);
    expected.push("../src/utils.rs");
    println!("expected: {}", expected.to_string_lossy());

    let expected = Url::from_file_path(expected.canonicalize().unwrap()).unwrap();
    assert_eq!(location.path()?.to_string(), expected.to_string());

    Ok(())
  }
}
