use std::borrow::Cow;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use asset_container::{self as assets, Asset, AssetManager, Progress};
use bytes::Bytes;
use normpath::PathExt;
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tokio_stream::Stream;
use tracing::debug;
use wick_oci_utils::OciOptions;

use crate::{normalize_path, Error};

#[derive(Debug, Clone)]
pub struct FetchableAssetReference<'a>(&'a AssetReference, OciOptions);

impl<'a> FetchableAssetReference<'a> {
  pub async fn bytes(&self) -> Result<Bytes, Error> {
    self.0.bytes(&self.1).await
  }
}

impl<'a> std::ops::Deref for FetchableAssetReference<'a> {
  type Target = AssetReference;

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

#[derive(Debug, Clone, serde::Serialize)]
#[must_use]
pub struct AssetReference {
  pub(crate) location: String,
  #[serde(skip)]
  pub(crate) cache_location: Arc<RwLock<Option<PathBuf>>>,
  #[serde(skip)]
  pub(crate) baseurl: Arc<RwLock<Option<PathBuf>>>,
}

impl FromStr for AssetReference {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::new(s))
  }
}

impl From<&str> for AssetReference {
  fn from(s: &str) -> Self {
    Self::new(s)
  }
}

impl std::fmt::Display for AssetReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.location)
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

  /// Embed [OciOptions] with an [AssetReference].
  #[must_use]
  pub fn with_options(&self, options: OciOptions) -> FetchableAssetReference<'_> {
    FetchableAssetReference(self, options)
  }

  /// Get the relative part of the path or return an error if the path does not exist within the base URL.
  pub fn get_relative_part(&self) -> Result<PathBuf, Error> {
    let path = self.path()?;
    let base_dir = self.baseurl().unwrap(); // safe to unwrap because this is a developer error if it panics.

    let base_dir = base_dir.normalize().map_err(|_| Error::NotFound(path.clone()))?;
    let mut base_dir = base_dir.as_path().to_string_lossy().to_string();
    if !base_dir.ends_with('/') {
      base_dir.push('/');
    }

    path.strip_prefix(&base_dir).map_or_else(
      |_e| Err(Error::FileEscapesRoot(path.clone(), base_dir)),
      |s| Ok(s.to_owned()),
    )
  }

  #[must_use]
  pub fn baseurl(&self) -> Option<PathBuf> {
    self.baseurl.read().clone()
  }

  pub fn path(&self) -> Result<PathBuf, Error> {
    self.resolve_path(true)
  }

  #[allow(clippy::option_if_let_else)]
  fn resolve_path(&self, use_cache: bool) -> Result<PathBuf, Error> {
    if let Some(cache_loc) = self.cache_location.read().as_ref() {
      if use_cache {
        return Ok(cache_loc.clone());
      }
    }
    if let Ok(url) = normalize_path(self.location.as_ref(), self.baseurl()) {
      Ok(url)
    } else if wick_oci_utils::is_oci_reference(self.location.as_str()) {
      Ok(PathBuf::from(&self.location))
    } else {
      Err(Error::Unresolvable(self.location.clone()))
    }
  }

  #[must_use]
  pub fn location(&self) -> &str {
    &self.location
  }

  #[must_use]
  pub fn is_directory(&self) -> bool {
    self.path().map_or(false, |path| path.is_dir())
  }

  pub async fn bytes(&self, options: &OciOptions) -> Result<Bytes, Error> {
    match self.fetch(options.clone()).await {
      Ok(bytes) => Ok(bytes.into()),
      Err(_err) => Err(Error::LoadError(self.path()?)),
    }
  }

  /// Check if the asset exists on disk.
  #[must_use]
  pub fn exists_locally(&self) -> bool {
    let path = self.resolve_path(false);
    path.is_ok() && path.unwrap().exists()
  }
}

impl Asset for AssetReference {
  type Options = OciOptions;

  #[allow(clippy::expect_used)]
  fn update_baseurl(&self, baseurl: &Path) {
    let baseurl = if baseurl.starts_with(".") {
      let mut path = std::env::current_dir().expect("failed to get current dir");
      path.push(baseurl);

      path
    } else {
      baseurl.to_owned()
    };

    *self.baseurl.write() = Some(baseurl);
  }

  fn fetch_with_progress(&self, _options: OciOptions) -> std::pin::Pin<Box<dyn Stream<Item = Progress> + Send + '_>> {
    unimplemented!()
  }

  fn fetch(
    &self,
    options: OciOptions,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<u8>, assets::Error>> + Send + Sync>> {
    let path = self.path();
    let location = self.location().to_owned();

    let cache_location = self.cache_location.clone();
    let exists = self.exists_locally();
    Box::pin(async move {
      if exists {
        let path = path.unwrap();
        if path.is_dir() {
          return Err(assets::Error::IsDirectory(path.clone()));
        }

        debug!(path = ?path, "fetching local asset");
        let mut file = tokio::fs::File::open(&path)
          .await
          .map_err(|err| assets::Error::FileOpen(path.clone(), err.to_string()))?;
        let mut bytes = Vec::new();

        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
      } else {
        let path = location;
        debug!(path = ?path, "fetching remote asset");
        let (cache_loc, bytes) = retrieve_remote(&path, options)
          .await
          .map_err(|err| assets::Error::RemoteFetch(path, err.to_string()))?;
        *cache_location.write() = Some(cache_loc);
        Ok(bytes)
      }
    })
  }

  fn name(&self) -> &str {
    self.location.as_str()
  }
}

async fn retrieve_remote(location: &str, options: OciOptions) -> Result<(PathBuf, Vec<u8>), Error> {
  let result = wick_oci_utils::package::pull(location, &options)
    .await
    .map_err(|e| Error::PullFailed(PathBuf::from(location), e.to_string()))?;
  let cache_location = result.base_dir.join(result.root_path);
  let bytes = tokio::fs::read(&cache_location)
    .await
    .map_err(|_| Error::LoadError(cache_location.clone()))?;
  Ok((cache_location, bytes))
}

impl AssetManager for AssetReference {
  type Asset = AssetReference;

  fn set_baseurl(&self, baseurl: &Path) {
    self.update_baseurl(baseurl);
  }

  fn assets(&self) -> assets::Assets<Self::Asset> {
    assets::Assets::new(vec![Cow::Borrowed(self)], 0)
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
    assert_eq!(location.path()?, expected);
    assert!((location.path()?).exists());
    Ok(())
  }

  #[test_logger::test]
  fn test_baseurl() -> Result<()> {
    let location = AssetReference::new("Cargo.toml");
    let mut root_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    root_project_dir.pop();
    root_project_dir.pop();
    root_project_dir.pop();

    location.set_baseurl(&root_project_dir);
    let mut expected = root_project_dir;
    expected.push("Cargo.toml");
    assert_eq!(location.path()?, expected);
    assert!((location.path()?).exists());

    Ok(())
  }

  #[test_logger::test]
  fn test_relative_with_baseurl() -> Result<()> {
    let location = AssetReference::new("../Cargo.toml");
    let mut root_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    root_project_dir.pop();
    root_project_dir.pop();

    location.set_baseurl(&root_project_dir);
    let mut expected = root_project_dir;
    expected.pop();
    expected.push("Cargo.toml");
    assert_eq!(location.path()?, expected);

    Ok(())
  }

  #[test_logger::test]
  fn test_relative_with_baseurl2() -> Result<()> {
    let location = AssetReference::new("../src/utils.rs");
    let mut crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    crate_dir.push("src");
    location.set_baseurl(&crate_dir);
    println!("crate_dir: {}", crate_dir.to_string_lossy());
    println!("actual: {:#?}", location);
    let mut expected = PathBuf::from(&crate_dir);
    expected.push("../src/utils.rs");
    println!("expected: {}", expected.to_string_lossy());

    let expected = expected.canonicalize()?;
    assert_eq!(location.path()?, expected);

    Ok(())
  }

  #[rstest::rstest]
  #[case("./files/assets/test.fake.wasm", Ok("files/assets/test.fake.wasm"))]
  #[case("./files/./assets/test.fake.wasm", Ok("files/assets/test.fake.wasm"))]
  #[case("./files/../files/assets/test.fake.wasm", Ok("files/assets/test.fake.wasm"))]
  fn test_path_normalization(#[case] path: &str, #[case] expected: Result<&str, Error>) -> Result<()> {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let testdata_dir = crate_dir.join("../../../tests/testdata");

    println!("base dir: {}", testdata_dir.display());
    println!("asset location: {}", path);
    let asset = AssetReference::new(path);
    asset.update_baseurl(&testdata_dir);

    let result = asset.get_relative_part();
    let expected = expected.map(|s| PathBuf::from(s.to_owned()));
    assert_eq!(result, expected);

    Ok(())
  }

  #[rstest::rstest]
  #[case(
    "./tests/testdata/files/assets/test.fake.wasm",
    Ok("tests/testdata/files/assets/test.fake.wasm")
  )]
  #[case(
    "./tests/../tests/testdata/files/./assets/test.fake.wasm",
    Ok("tests/testdata/files/assets/test.fake.wasm")
  )]
  #[case(
    "./tests/./testdata/./files/../files/assets/test.fake.wasm",
    Ok("tests/testdata/files/assets/test.fake.wasm")
  )]
  fn test_baseurl_normalization(#[case] path: &str, #[case] expected: Result<&str, Error>) -> Result<()> {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let ws_dir = crate_dir.join("../../../");
    let base_dir = ws_dir.join("tests/./testdata/../../tests/./testdata/../..");

    println!("base dir: {}", base_dir.display());
    println!("asset location: {}", path);
    let asset = AssetReference::new(path);
    asset.update_baseurl(&base_dir);

    let result = asset.get_relative_part();
    let expected = expected.map(|s| PathBuf::from(s.to_owned()));
    assert_eq!(result, expected);

    Ok(())
  }

  #[rstest::rstest]
  #[case("/tests/testdata/files/assets/test.fake.wasm", Ok("files/assets/test.fake.wasm"))]
  #[case(
    "/tests/../tests/testdata/files/./assets/test.fake.wasm",
    Ok("files/assets/test.fake.wasm")
  )]
  #[case(
    "/tests/./testdata/./files/../files/assets/test.fake.wasm",
    Ok("files/assets/test.fake.wasm")
  )]
  fn test_path_normalization_absolute(#[case] path: &str, #[case] expected: Result<&str, Error>) -> Result<()> {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let ws_dir = crate_dir.join("../../../");
    let testdata_dir = ws_dir.join("tests/./testdata/../../tests/./testdata");
    let path = format!("{}/{}", ws_dir.display(), path);

    println!("base dir: {}", testdata_dir.display());
    println!("asset location: {}", path);
    let asset = AssetReference::new(path);
    asset.update_baseurl(&testdata_dir);

    let result = asset.get_relative_part();
    let expected = expected.map(|s| PathBuf::from(s.to_owned()));
    assert_eq!(result, expected);

    Ok(())
  }
}
