use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use assets::AssetManager;
use oci_distribution::client::{Client, ClientConfig, ImageLayer};
use oci_distribution::manifest::{OciDescriptor, OciImageManifest};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use serde::{Deserialize, Serialize};
use sha256::digest;
use tokio::fs;
use url::Url;
use wick_config::WickConfiguration;

use crate::annotations::{self, Annotations};
use crate::{media_types, Error};

/// Represents a single file in a Wick package.
#[derive(Debug)]
pub struct WickFile {
  path: PathBuf,
  hash: String,
  media_type: String,
  contents: Vec<u8>,
}

impl WickFile {
  /// Get path for the file.
  #[must_use]
  pub fn path(&self) -> &PathBuf {
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
}

/// Represents a Wick package, including its files and metadata.
#[derive(Debug)]
pub struct WickPackage {
  kind: WickPackageKind,
  #[allow(dead_code)]
  name: String,
  #[allow(dead_code)]
  version: String,
  files: Vec<WickFile>,
  annotations: Annotations,
}

#[derive(Debug, Serialize, Deserialize)]
struct WickConfig {
  kind: WickPackageKind,
}

/// Represents the kind of Wick package.
/// This is used to determine how to handle the package.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WickPackageKind {
  /// A Wick application package.
  APPLICATION,
  /// A Wick component package.
  COMPONENT,
}

impl WickPackage {
  /// Creates a new WickPackage from the provided path.
  ///
  /// The provided path can be a file or directory. If it is a directory, the WickPackage will be created
  /// based on the files within the directory.
  pub async fn from_path(path: &Path) -> Result<Self, Error> {
    //add check to see if its a path or directory and call appropriate api to find files based on that.
    if path.is_dir() {
      return Err(Error::Directory(path.to_path_buf()));
    }

    let options = wick_config::common::FetchOptions::default();
    let config = WickConfiguration::fetch(wick_config::path_to_url(path, None)?, options).await?;
    if !matches!(config, WickConfiguration::App(_) | WickConfiguration::Component(_)) {
      return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string()));
    }
    let full_path = tokio::fs::canonicalize(path)
      .await
      .map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;
    let parent_dir = full_path
      .parent()
      .map_or_else(|| PathBuf::from("/"), |v| v.to_path_buf());

    let (kind, name, version, annotations, parent_dir, media_type) = match &config {
      WickConfiguration::App(app_config) => {
        let name = app_config.name();
        let version = app_config.version();
        let annotations = Annotations::from(app_config.metadata());
        let media_type = media_types::APPLICATION;
        let kind = WickPackageKind::APPLICATION;
        (kind, name, version, annotations, parent_dir, media_type)
      }
      WickConfiguration::Component(component_config) => {
        let name = component_config.name().clone().ok_or(Error::NoName)?;
        let version = component_config.version();
        let annotations = Annotations::from(component_config.metadata());
        let media_type = media_types::COMPONENT;
        let kind = WickPackageKind::COMPONENT;
        (kind, name, version, annotations, parent_dir, media_type)
      }
      _ => return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string())),
    };

    let mut assets = config.assets();
    let mut wick_files: Vec<WickFile> = Vec::new();

    let root_bytes = fs::read(path)
      .await
      .map_err(|e| Error::ReadFile(path.to_path_buf(), e))?;
    let root_hash = format!("sha256:{}", digest(root_bytes.as_slice()));

    let root_file = WickFile {
      path: PathBuf::from(path.file_name().unwrap()),
      hash: root_hash,
      media_type: media_type.to_owned(),
      contents: root_bytes,
    };

    wick_files.push(root_file);

    //populate wick_files
    for asset in assets.iter() {
      let location = asset.location(); // the path specified in the config
      let asset_url = asset.path()?; // the resolved, abolute path relative to the config location.

      let path = get_relative_path(&parent_dir, &asset_url)?;

      let options = wick_config::common::FetchOptions::default();
      let media_type: &str;

      match path.extension().and_then(|os_str| os_str.to_str()) {
        Some("yaml" | "yml" | "wick") => {
          let config = WickConfiguration::fetch(asset_url, options.clone()).await;
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
      let wick_file = WickFile {
        path: PathBuf::from(location),
        hash,
        media_type: media_type.to_owned(),
        contents: file_bytes.to_vec(),
      };
      wick_files.push(wick_file);
    }

    Ok(Self {
      kind,
      name: name.clone(),
      version: version.clone(),
      files: wick_files,
      annotations,
    })
  }

  #[must_use]
  /// Returns a list of the files contained within the WickPackage.
  pub fn list_files(&self) -> Vec<&WickFile> {
    self.files.iter().collect()
  }

  /// Pushes the WickPackage to a specified registry using the provided reference, username, and password.
  ///
  /// The username and password are optional. If not provided, the function falls back to anonymous authentication.
  pub async fn push(
    &self,
    reference: &str,
    username: Option<&str>,
    password: Option<&str>,
    insecure: Option<bool>,
  ) -> Result<String, Error> {
    let insecure = insecure.unwrap_or(false);

    let image_config = WickConfig { kind: self.kind };

    let image_config_contents = serde_json::to_string(&image_config).unwrap();

    let image_config = oci_distribution::client::Config {
      data: image_config_contents.as_bytes().to_vec(),
      media_type: media_types::CONFIG.to_owned(),
      annotations: None,
    };

    let mut image_layer_descriptors: Vec<OciDescriptor> = Vec::new();
    let mut image_layers: Vec<ImageLayer> = Vec::new();

    for file in self.files.iter() {
      let mut annotations_map: HashMap<String, String> = HashMap::new();

      annotations_map.insert(annotations::TITLE.to_owned(), file.path.display().to_string());

      let image_layer = ImageLayer {
        data: file.contents.clone(),
        media_type: file.media_type.clone(),
        annotations: None,
      };

      let image_layer_descriptor = OciDescriptor {
        media_type: file.media_type.clone(),
        digest: file.hash.clone(),
        size: file.contents.len() as i64,
        annotations: Some(annotations_map),
        urls: None,
      };

      image_layer_descriptors.push(image_layer_descriptor);
      image_layers.push(image_layer);
    }

    let image_annotations: HashMap<String, String> = self.annotations.inner().clone();

    let image_manifest = OciImageManifest {
      schema_version: 2,
      config: OciDescriptor {
        media_type: image_config.media_type.clone(),
        digest: format!("sha256:{}", digest(image_config_contents.clone())),
        size: image_config.data.clone().len() as i64,
        annotations: None,
        urls: None,
      },
      layers: image_layer_descriptors,
      media_type: Some(media_types::MANIFEST.to_owned()),
      annotations: Some(image_annotations),
    };

    let client_config = ClientConfig {
      protocol: match insecure {
        true => oci_distribution::client::ClientProtocol::Http,
        false => oci_distribution::client::ClientProtocol::Https,
      },
      ..Default::default()
    };

    let auth = match (username.as_ref(), password.as_ref()) {
      (Some(username), Some(password)) => RegistryAuth::Basic((*username).to_owned(), (*password).to_owned()),
      _ => {
        println!("Both username and password must be supplied. Falling back to anonymous auth");
        RegistryAuth::Anonymous
      }
    };

    let mut client = Client::new(client_config);
    let image_ref_result = Reference::from_str(reference);
    let image_ref = match image_ref_result {
      Ok(image_ref) => {
        println!("Pushing package to registry: {}", image_ref);
        image_ref
      }
      Err(_) => {
        return Err(Error::InvalidReference(reference.to_owned()));
      }
    };

    let result = client
      .push(
        &image_ref,
        &image_layers,
        image_config,
        &auth,
        Some(image_manifest.clone()),
      )
      .await;

    match result {
      Ok(push_response) => {
        println!("Image successfully pushed to the registry.");
        println!("Config URL: {}", push_response.config_url);
        println!("Manifest URL: {}", push_response.manifest_url);
        Ok(push_response.manifest_url)
      }
      Err(e) => {
        println!("Push failed: {}", e);
        println!("Push failed: {}", image_manifest);
        Err(Error::PushFailed(e.to_string()))
      }
    }
  }

  /// This function pulls a WickPackage from a specified registry using the provided reference, username, and password.
  pub async fn pull(
    reference: &str,
    username: Option<&str>,
    password: Option<&str>,
    insecure: Option<bool>,
  ) -> Result<Self, Error> {
    let insecure = insecure.unwrap_or(false);
    let client_config = ClientConfig {
      protocol: match insecure {
        true => oci_distribution::client::ClientProtocol::Http,
        false => oci_distribution::client::ClientProtocol::Https,
      },
      ..Default::default()
    };

    let mut client = Client::new(client_config);
    let image_ref_result = Reference::from_str(reference);
    let image_ref = match image_ref_result {
      Ok(image_ref) => {
        println!("Pulling package from registry: {}", image_ref);
        image_ref
      }
      Err(_) => {
        return Err(Error::InvalidReference(reference.to_owned()));
      }
    };

    let auth = match (username.as_ref(), password.as_ref()) {
      (Some(username), Some(password)) => RegistryAuth::Basic((*username).to_owned(), (*password).to_owned()),
      _ => {
        println!("Both username and password must be supplied. Falling back to anonymous auth");
        RegistryAuth::Anonymous
      }
    };

    let accepted_media_types = vec![
      media_types::CONFIG,
      media_types::MANIFEST,
      media_types::APPLICATION,
      media_types::COMPONENT,
      media_types::TESTS,
      media_types::TYPES,
      media_types::WASM,
      media_types::OTHER,
    ];

    let result = client.pull(&image_ref, &auth, accepted_media_types).await;

    let image_data = match result {
      Ok(pull_response) => {
        println!("Image successfully pulled from the registry.");
        pull_response
      }
      Err(e) => {
        println!("Pull failed: {}", e);
        return Err(Error::PullFailed(e.to_string()));
      }
    };

    // serialize the config data from bytes available via image_data.config.data.clone()
    let wick_config: WickConfig =
      serde_json::from_slice(&image_data.config.data).map_err(|e| Error::InvalidJson("Image config", e))?;
    let kind = wick_config.kind;

    let download_dir = if kind == WickPackageKind::APPLICATION {
      PathBuf::from("./")
    } else {
      match create_directory_structure(reference).await {
        Ok(path) => {
          println!("Directory created successfully: {}", path.display());
          path
        }
        Err(e) => return Err(Error::DirectoryCreationFailed(e.to_string())),
      }
    };

    let mut root_file: Option<String> = None;

    for layer in image_data.layers {
      let layer_title = layer
        .annotations
        .and_then(|v| v.get(annotations::TITLE).cloned())
        .ok_or(Error::NoTitle)?;
      let layer_path = download_dir.join(&layer_title);
      let parent_dir = layer_path.parent().ok_or(Error::InvalidLayerPath(layer_path.clone()))?;
      //create any subdirectories if they don't exist.
      tokio::fs::create_dir_all(parent_dir)
        .await
        .map_err(|e| Error::CreateDir(parent_dir.to_path_buf(), e))?;
      tokio::fs::write(&layer_path, &layer.data)
        .await
        .map_err(|e| Error::WriteFile(layer_path, e))?;

      if layer.media_type == media_types::APPLICATION || layer.media_type == media_types::COMPONENT {
        root_file = Some(layer_title);
      }
    }

    let root_file = root_file.ok_or_else(|| Error::PackageReadFailed("No root file found".to_owned()))?;
    println!("Root file: {}", root_file);

    let package = Self::from_path(&download_dir.join(Path::new(&root_file))).await;

    match package {
      Ok(package) => Ok(package),
      Err(e) => Err(Error::PackageReadFailed(e.to_string())),
    }
  }
}

fn get_relative_path(base_dir: &PathBuf, url: &Url) -> Result<PathBuf, Error> {
  if url.scheme() == "file" {
    // Get the prefix of the path that matches the base directory
    let path = url
      .to_file_path()
      .map_err(|_e| Error::InvalidFileLocation(url.to_string()))?;
    let relative_part = path
      .strip_prefix(base_dir)
      .map_err(|_| Error::InvalidFileLocation(url.to_string()))?;

    // Return the relative path
    Ok(relative_part.to_path_buf())
  } else {
    Err(Error::InvalidFileLocation(url.to_string()))
  }
}

async fn create_directory_structure(input: &str) -> Result<PathBuf, Error> {
  // Parse the input reference
  let image_ref_result = Reference::from_str(input);
  let image_ref = match image_ref_result {
    Ok(image_ref) => image_ref,
    Err(_) => {
      return Err(Error::InvalidReference(input.to_owned()));
    }
  };

  let registry = image_ref.registry().split(':').collect::<Vec<&str>>()[0];
  let org = image_ref.repository().split('/').collect::<Vec<&str>>()[0];
  let repo = image_ref.repository().split('/').collect::<Vec<&str>>()[1];
  let version = image_ref.tag().ok_or(Error::NoName)?;

  // put these 4 variables in a vector called parts
  let parts = vec![registry, org, repo, version];

  if parts.len() != 4 {
    return Err(Error::InvalidReference(input.to_owned()));
  }

  // Create the wick_components directory if it doesn't exist
  let base_dir = Path::new("./wick_components");
  fs::create_dir_all(&base_dir)
    .await
    .map_err(|e| Error::CreateDir(base_dir.to_path_buf(), e))?;

  // Create the required subdirectories
  let target_dir = base_dir.join(registry).join(org).join(repo).join(version);
  fs::create_dir_all(&target_dir)
    .await
    .map_err(|e| Error::CreateDir(target_dir.clone(), e))?;

  println!("Directory created: {}", target_dir.display());

  Ok(target_dir)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ensure_relative_path() {
    let parent_dir = PathBuf::from("/candlecorp/wick/crates/wick/wick-package/tests/files");
    let url =
      Url::from_file_path("/candlecorp/wick/crates/wick/wick-package/tests/files/assets/test.fake.wasm").unwrap();

    let result = get_relative_path(&parent_dir, &url);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("assets/test.fake.wasm"));
  }

  #[tokio::test]
  async fn test_create_directory_structure() {
    let input = "localhost:8888/test/integration:0.0.3";
    let expected_dir = Path::new("./wick_components/localhost/test/integration/0.0.3");
    let result = create_directory_structure(input).await.unwrap();
    assert_eq!(result, expected_dir);

    let input = "example.com/myorg/myrepo:1.0.0";
    let expected_dir = Path::new("./wick_components/example.com/myorg/myrepo/1.0.0");
    let result = create_directory_structure(input).await.unwrap();
    assert_eq!(result, expected_dir);
  }
}
