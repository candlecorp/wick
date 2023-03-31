//! This is the documentation for the `wick-package` crate.
//!
//! The `wick-package` crate provides functionality for handling Wick packages,
//! including reading, writing, and manipulating package data.

// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow()]

mod annotations;
mod error;
mod media_types;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use assets::AssetManager;
pub use error::Error;
use oci_distribution::client::{Client, ClientConfig, ImageLayer};
use oci_distribution::manifest::{OciDescriptor, OciImageManifest};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use sha256::digest;
use wick_config::WickConfiguration;

/// Represents a single file in a Wick package.
#[derive(Debug)]
pub struct WickFile {
  path: PathBuf,
  hash: String,
  media_type: String,
  contents: Vec<u8>,
}

/// Represents a Wick package, including its files and metadata.
#[derive(Debug)]
pub struct WickPackage {
  #[allow(dead_code)]
  name: String,
  #[allow(dead_code)]
  version: String,
  files: Vec<WickFile>,
  annotations: HashMap<String, String>,
}

impl WickPackage {
  /// Creates a new WickPackage from the provided path.
  ///
  /// The provided path can be a file or directory. If it is a directory, the WickPackage will be created
  /// based on the files within the directory.
  pub async fn from_path(path: &Path) -> Result<Self, Error> {
    //add check to see if its a path or directory and call appropriate api to find files based on that.
    if path.is_dir() {
      return Err(Error::Directory(path.to_string_lossy().to_string()));
    }

    let options = wick_config::common::FetchOptions::default();
    let config = WickConfiguration::fetch(path.to_str().unwrap(), options).await.unwrap();
    if !matches!(config, WickConfiguration::App(_) | WickConfiguration::Component(_)) {
      return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string()));
    }

    let (name, version, annotations, parent_dir) = if matches!(&config, WickConfiguration::App(_)) {
      let app_config = config.clone().try_app_config().unwrap();
      let name = app_config.name();
      let version = app_config.version();
      let annotations = HashMap::from(&app_config.metadata());
      let full_path = tokio::fs::canonicalize(path).await.unwrap();
      let parent_dir = full_path.parent().unwrap().to_path_buf();
      (name, version, annotations, parent_dir)
    } else if matches!(config, WickConfiguration::Component(_)) {
      let component_config = config.clone().try_component_config().unwrap();
      let name = component_config.name().clone().unwrap();
      let version = component_config.version();
      let annotations = HashMap::from(&component_config.metadata());
      let full_path = tokio::fs::canonicalize(path).await.unwrap();
      let parent_dir = full_path.parent().unwrap().to_path_buf();
      (name, version, annotations, parent_dir)
    } else {
      return Err(Error::InvalidWickConfig(path.to_string_lossy().to_string()));
    };

    let mut assets = config.assets();
    let mut wick_files: Vec<WickFile> = Vec::new();

    //populate wick_files
    for asset in assets.iter() {
      let location = asset.location(); // the path specified in the config
      let path = asset.path(); // the resolved, abolute path relative to the config location.

      ensure_relative_path(&parent_dir, &path).unwrap();

      let options = wick_config::common::FetchOptions::default();
      let media_type: &str;

      match path.extension().and_then(|os_str| os_str.to_str()) {
        Some("yaml" | "yml" | "wick") => {
          let config = WickConfiguration::fetch(path.to_str().unwrap(), options.clone()).await;
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

      let file_bytes = asset.bytes(&options).await.unwrap();
      let hash = format!("sha256:{}", digest(String::from_utf8(file_bytes.to_vec()).unwrap()));
      let wick_file = WickFile {
        path: PathBuf::from(location),
        hash: hash,
        media_type: media_type.to_owned(),
        contents: file_bytes.to_vec(),
      };
      wick_files.push(wick_file);
    }

    Ok(Self {
      name: name.clone(),
      version: version.clone(),
      files: wick_files,
      annotations: annotations,
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
  pub async fn push(&self, reference: &str, username: Option<&str>, password: Option<&str>) -> Result<String, Error> {
    let image_config_contents = "{}"; //this is the config file for the oci image

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

    let mut image_annotations: HashMap<String, String> = HashMap::new();
    for (key, value) in self.annotations.iter() {
      let new_key = match key.as_str() {
        "version" => annotations::VERSION,
        "icon" => annotations::ICON,
        "type" => annotations::TYPE,
        "authors" => annotations::AUTHORS,
        "vendors" => annotations::VENDORS,
        "description" => annotations::DESCRIPTION,
        "documentation" => annotations::DOCUMENTATION,
        "licenses" => annotations::LICENSES,
        _ => key.as_str(),
      };
      image_annotations.insert(new_key.to_owned(), value.to_string());
    }

    let image_manifest = OciImageManifest {
      schema_version: 2,
      config: OciDescriptor {
        media_type: image_config.media_type.clone(),
        digest: format!("sha256:{}", digest(image_config_contents.to_owned())),
        size: image_config.data.clone().len() as i64,
        annotations: None,
        urls: None,
      },
      layers: image_layer_descriptors,
      media_type: Some(media_types::MANIFEST.to_owned()),
      annotations: Some(image_annotations),
    };

    let client_config = ClientConfig {
      protocol: oci_distribution::client::ClientProtocol::Https,
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
      .push(&image_ref, &image_layers, image_config, &auth, Some(image_manifest))
      .await;

    match result {
      Ok(push_response) => {
        println!("Image successfully pushed to the registry.");
        println!("Config URL: {}", push_response.config_url);
        println!("Manifest URL: {}", push_response.manifest_url);
        Ok(push_response.manifest_url)
      }
      Err(e) => {
        eprintln!("Failed to push the package: {}", e);
        std::process::exit(1);
      }
    }
  }
}

fn ensure_relative_path(base_dir: &PathBuf, path: &Path) -> Result<PathBuf, Error> {
  // Get the prefix of the path that matches the base directory
  let prefix = path
    .strip_prefix(base_dir)
    .map_err(|_| Error::InvalidFileLocation(path.to_string_lossy().to_string()))?;

  // Check if the prefix is empty, indicating that the path is not going above the base directory
  if prefix.as_os_str().is_empty() {
    // The path is a valid relative path inside the base directory
    Ok(prefix.to_path_buf())
  } else {
    // The path is going above the base directory
    return Err(Error::InvalidFileLocation(path.to_string_lossy().to_string()));
  }
}
