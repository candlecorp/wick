use std::collections::HashSet;
use std::future::Future;
use std::path::{Path, PathBuf};

use asset_container::{Asset, AssetFlags, AssetManager, Assets};
use sha256::digest;
use tokio::fs;
use tracing::trace;
use wick_config::config::{AssetReference, RegistryConfig};
use wick_config::WickConfiguration;
use wick_oci_utils::package::annotations::Annotations;
use wick_oci_utils::package::{media_types, PackageFile};
use wick_oci_utils::OciOptions;

use crate::utils::{create_tar_gz, metadata_to_annotations};
use crate::Error;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

fn process_assets(
  mut seen_assets: HashSet<PathBuf>,
  assets: Assets<AssetReference>,
  root_parent_dir: PathBuf,
  parent_dir: PathBuf,
) -> BoxFuture<Result<(HashSet<PathBuf>, Vec<PackageFile>), Error>> {
  let task = async move {
    let mut wick_files: Vec<PackageFile> = Vec::new();
    for asset in assets.iter() {
      if asset.get_asset_flags() == AssetFlags::Lazy {
        continue;
      }
      let pdir = parent_dir.clone();
      asset.update_baseurl(&pdir);
      let asset_path = asset.path()?; // the resolved, abolute path relative to the config location.
      if seen_assets.contains(&asset_path) {
        continue;
      }
      seen_assets.insert(asset_path.clone());

      let relative_path = asset_path.strip_prefix(&root_parent_dir).unwrap_or(&asset_path);

      let options = wick_config::FetchOptions::default();
      let media_type: &str;

      let new_parent_dir = asset_path
        .parent()
        .map_or_else(|| PathBuf::from("/"), |v| v.to_path_buf());

      match relative_path.extension().and_then(|os_str| os_str.to_str()) {
        Some("yaml" | "yml" | "wick") => {
          let config = WickConfiguration::fetch(asset_path.to_string_lossy(), options.clone()).await;
          match config {
            Ok(WickConfiguration::App(config)) => {
              media_type = media_types::APPLICATION;
              let app_assets = config.assets();
              let (newly_seen, app_files) =
                process_assets(seen_assets, app_assets, root_parent_dir.clone(), new_parent_dir.clone()).await?;
              seen_assets = newly_seen;
              wick_files.extend(app_files);
            }
            Ok(WickConfiguration::Component(config)) => {
              media_type = media_types::COMPONENT;
              let component_assets = config.assets();
              let (newly_seen, component_files) = process_assets(
                seen_assets,
                component_assets,
                root_parent_dir.clone(),
                new_parent_dir.clone(),
              )
              .await?;
              seen_assets = newly_seen;
              wick_files.extend(component_files);
            }
            Ok(WickConfiguration::Tests(_)) => {
              media_type = media_types::TESTS;
            }
            Ok(WickConfiguration::Types(_)) => {
              media_type = media_types::TYPES;
            }
            Err(_) => {
              media_type = media_types::OTHER;
            }
          }
        }
        Some("wasm") => {
          media_type = media_types::WASM;
        }
        _ => {
          media_type = media_types::OTHER;
        }
      }

      if asset.exists_locally() {
        let file_bytes = asset.bytes(&options).await?;
        let hash = format!("sha256:{}", digest(file_bytes.as_ref()));
        let wick_file = PackageFile::new(relative_path.to_path_buf(), hash, media_type.to_owned(), file_bytes);
        wick_files.push(wick_file);
      }
    }

    Ok((seen_assets, wick_files))
  };
  Box::pin(task)
}

/// Represents a Wick package, including its files and metadata.
#[derive(Debug, Clone)]
pub struct WickPackage {
  kind: wick_oci_utils::WickPackageKind,
  name: String,
  version: String,
  files: Vec<PackageFile>,
  annotations: Annotations,
  absolute_path: PathBuf,
  registry: Option<RegistryConfig>,
  root: String,
}

impl WickPackage {
  /// Creates a new WickPackage from the provided path.
  ///
  /// The provided path can be a file or directory. If it is a directory, the WickPackage will be created
  /// based on the files within the directory.
  #[allow(clippy::too_many_lines)]
  pub async fn from_path(path: &Path) -> Result<Self, Error> {
    //add check to see if its a path or directory and call appropriate api to find files based on that.
    if path.is_dir() {
      return Err(Error::Directory(path.to_path_buf()));
    }

    let options = wick_config::FetchOptions::default();
    let config = WickConfiguration::fetch(path.to_string_lossy(), options).await?;
    if !matches!(
      config,
      WickConfiguration::App(_) | WickConfiguration::Component(_) | WickConfiguration::Types(_)
    ) {
      return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string()));
    }
    let full_path = tokio::fs::canonicalize(path)
      .await
      .map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;
    let parent_dir = full_path
      .parent()
      .map_or_else(|| PathBuf::from("/"), |v| v.to_path_buf());
    let extra_files;

    let (kind, name, version, annotations, parent_dir, media_type, registry) = match &config {
      WickConfiguration::App(config) => {
        let name = config.name().to_owned();
        let version = config.version();
        let annotations = metadata_to_annotations(&config.metadata());
        let media_type = media_types::APPLICATION;
        let kind = wick_oci_utils::WickPackageKind::APPLICATION;

        extra_files = config.package_files().to_owned();

        let registry = config.package().and_then(|package| package.registry().cloned());

        (kind, name, version, annotations, parent_dir, media_type, registry)
      }
      WickConfiguration::Component(config) => {
        let name = config.name().cloned().ok_or(Error::NoName)?;
        let version = config.version();
        let annotations = metadata_to_annotations(&config.metadata());
        let media_type = media_types::COMPONENT;
        let kind = wick_oci_utils::WickPackageKind::COMPONENT;

        extra_files = config.package_files().map_or_else(Vec::new, |files| files.to_owned());

        let registry = config.package().and_then(|package| package.registry().cloned());

        (kind, name, version, annotations, parent_dir, media_type, registry)
      }
      WickConfiguration::Types(config) => {
        let name = config.name().cloned().ok_or(Error::NoName)?;
        let version = config.version();
        let annotations = metadata_to_annotations(&config.metadata());
        let media_type = media_types::TYPES;
        let kind = wick_oci_utils::WickPackageKind::TYPES;

        extra_files = config.package_files().map_or_else(Vec::new, |files| files.to_owned());

        let registry = config.package().and_then(|package| package.registry().cloned());

        (kind, name, version, annotations, parent_dir, media_type, registry)
      }
      _ => return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string())),
    };

    let assets = config.assets();
    let mut wick_files: Vec<PackageFile> = Vec::new();

    let root_bytes = fs::read(path)
      .await
      .map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;
    let root_hash = format!("sha256:{}", digest(root_bytes.as_slice()));

    let root_file = PackageFile::new(
      PathBuf::from(path.file_name().unwrap()),
      root_hash,
      media_type.to_owned(),
      root_bytes.into(),
    );

    let root_file_path = path.file_name().unwrap().to_string_lossy().to_string();
    wick_files.push(root_file);

    //if length of extra_files is greater than 0, then we need create a tar of all the files
    //and add it to the files list.
    if !extra_files.is_empty() {
      let gz_bytes = create_tar_gz(extra_files, &parent_dir).await?;

      let tar_hash = format!("sha256:{}", digest(gz_bytes.as_slice()));
      let tar_file = PackageFile::new(
        PathBuf::from("extra_files.tar.gz"),
        tar_hash,
        media_types::TARGZ.to_owned(),
        gz_bytes.into(),
      );
      wick_files.push(tar_file);
    }

    //populate wick_files
    let (_, return_assets) = process_assets(Default::default(), assets, parent_dir.clone(), parent_dir.clone()).await?;
    //merge return assets  vector to wick_files
    wick_files.extend(return_assets);
    trace!(files = ?wick_files.iter().map(|f| f.path()).collect::<Vec<_>>(),
      "package files"
    );

    Ok(Self {
      kind,
      name: name.clone(),
      version: version.clone(),
      files: wick_files,
      annotations,
      absolute_path: full_path,
      registry,
      root: root_file_path,
    })
  }

  #[must_use]
  /// Returns a list of the files contained within the WickPackage.
  pub fn list_files(&self) -> Vec<&PackageFile> {
    self.files.iter().collect()
  }

  #[must_use]
  /// Returns a list of the files contained within the WickPackage.
  pub fn path(&self) -> &PathBuf {
    &self.absolute_path
  }

  #[must_use]
  /// Returns the reference.
  pub fn registry_reference(&self) -> Option<String> {
    self
      .registry
      .as_ref()
      .map(|r| format!("{}/{}/{}:{}", r.host(), r.namespace(), self.name, self.version))
  }

  #[must_use]
  /// Returns an OCI URL with the specified tag.
  pub fn tagged_reference(&self, tag: &str) -> Option<String> {
    self
      .registry
      .as_ref()
      .map(|r| format!("{}/{}/{}:{}", r.host(), r.namespace(), self.name, tag))
  }

  #[must_use]
  /// Returns the registry.
  pub fn registry(&self) -> Option<&RegistryConfig> {
    self.registry.as_ref()
  }

  /// Pushes the WickPackage to a specified registry using the provided reference, username, and password.
  ///
  /// The username and password are optional. If not provided, the function falls back to anonymous authentication.
  pub async fn push(&mut self, reference: &str, options: &OciOptions) -> Result<String, Error> {
    let config = wick_oci_utils::WickOciConfig {
      kind: self.kind,
      root: self.root.clone(),
    };
    let image_config_contents = serde_json::to_string(&config).unwrap();
    let files = self.files.drain(..).collect();

    let push_response = wick_oci_utils::package::push(
      reference,
      image_config_contents,
      files,
      self.annotations.clone(),
      options,
    )
    .await?;

    Ok(push_response.manifest_url)
  }

  /// This function pulls a WickPackage from a specified registry using the provided reference, username, and password.
  pub async fn pull(reference: &str, options: &OciOptions) -> Result<Self, Error> {
    let result = wick_oci_utils::package::pull(reference, options).await?;

    let package = Self::from_path(&result.base_dir.join(Path::new(&result.root_path))).await;

    match package {
      Ok(package) => Ok(package),
      Err(e) => Err(Error::PackageReadFailed(e.to_string())),
    }
  }
}
