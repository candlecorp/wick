use std::path::{Path, PathBuf};

use asset_container::{Asset, AssetFlags, AssetManager};
use sha256::digest;
use tokio::fs;
use wick_config::WickConfiguration;
use wick_oci_utils::package::annotations::Annotations;
use wick_oci_utils::package::{media_types, PackageFile};
use wick_oci_utils::OciOptions;

use crate::utils::{create_tar_gz, metadata_to_annotations};
use crate::{Error, WickPackageKind};

/// Represents a Wick package, including its files and metadata.
#[derive(Debug, Clone)]
pub struct WickPackage {
  #[allow(unused)]
  kind: WickPackageKind,
  #[allow(dead_code)]
  name: String,
  #[allow(dead_code)]
  version: String,
  files: Vec<PackageFile>,
  #[allow(unused)]
  annotations: Annotations,
  #[allow(unused)]
  absolute_path: PathBuf,
  #[allow(unused)]
  registry_reference: Option<String>,
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

    let registry_reference;

    let options = wick_config::FetchOptions::default();
    let config = WickConfiguration::fetch(&path.to_string_lossy(), options).await?;
    if !matches!(config, WickConfiguration::App(_) | WickConfiguration::Component(_)) {
      return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string()));
    }
    let full_path = tokio::fs::canonicalize(path)
      .await
      .map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;
    let parent_dir = full_path
      .parent()
      .map_or_else(|| PathBuf::from("/"), |v| v.to_path_buf());
    let extra_files;

    let (kind, name, version, annotations, parent_dir, media_type) = match &config {
      WickConfiguration::App(config) => {
        let name = config.name();
        let version = config.version();
        let annotations = metadata_to_annotations(config.metadata());
        let media_type = media_types::APPLICATION;
        let kind = WickPackageKind::APPLICATION;

        extra_files = config.package_files().to_owned();

        registry_reference = config.package.as_ref().and_then(|package| {
          package
            .registry
            .as_ref()
            .map(|registry| format!("{}/{}/{}:{}", registry.registry, registry.namespace, name, version))
        });

        (kind, name, version, annotations, parent_dir, media_type)
      }
      WickConfiguration::Component(config) => {
        let name = config.name().clone().ok_or(Error::NoName)?;
        let version = config.version();
        let annotations = metadata_to_annotations(config.metadata());
        let media_type = media_types::COMPONENT;
        let kind = WickPackageKind::COMPONENT;

        extra_files = config.package_files().map_or_else(Vec::new, |files| files.to_owned());

        registry_reference = config.package.as_ref().and_then(|package| {
          package
            .registry
            .as_ref()
            .map(|registry| format!("{}/{}/{}:{}", registry.registry, registry.namespace, name, version))
        });
        (kind, name, version, annotations, parent_dir, media_type)
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
    for asset in assets.iter() {
      if asset.get_asset_flags() == AssetFlags::Lazy {
        continue;
      }
      asset.update_baseurl(&parent_dir);
      let asset_path = asset.path()?; // the resolved, abolute path relative to the config location.

      let path = asset.get_relative_part()?;

      let options = wick_config::FetchOptions::default();
      let media_type: &str;

      match path.extension().and_then(|os_str| os_str.to_str()) {
        Some("yaml" | "yml" | "wick") => {
          let config = WickConfiguration::fetch(asset_path.to_string_lossy(), options.clone()).await;
          match config {
            Ok(WickConfiguration::App(_)) => {
              media_type = media_types::APPLICATION;
            }
            Ok(WickConfiguration::Component(_)) => {
              media_type = media_types::COMPONENT;
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

      let file_bytes = asset.bytes(&options).await?;
      let hash = format!("sha256:{}", digest(file_bytes.as_ref()));
      let wick_file = PackageFile::new(path, hash, media_type.to_owned(), file_bytes);
      wick_files.push(wick_file);
    }

    Ok(Self {
      kind,
      name: name.clone(),
      version: version.clone(),
      files: wick_files,
      annotations,
      absolute_path: full_path,
      registry_reference,
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
    self.registry_reference.as_ref().and_then(|reference| {
      if reference.is_empty() {
        None
      } else {
        Some(reference.clone())
      }
    })
  }

  /// Pushes the WickPackage to a specified registry using the provided reference, username, and password.
  ///
  /// The username and password are optional. If not provided, the function falls back to anonymous authentication.
  pub async fn push(&mut self, reference: &str, options: &OciOptions) -> Result<String, Error> {
    let config = crate::WickConfig { kind: self.kind };
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

    println!("Image successfully pushed to the registry.");
    println!("Config URL: {}", push_response.config_url);
    println!("Manifest URL: {}", push_response.manifest_url);
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
