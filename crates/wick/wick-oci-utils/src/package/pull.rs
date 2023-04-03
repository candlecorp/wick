use std::path::PathBuf;
use std::str::FromStr;

use oci_distribution::client::ClientConfig;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

use super::{annotations, media_types};
use crate::utils::create_directory_structure;
use crate::Error;

/// Result of a pull operation.
#[derive(Debug, Clone)]
pub struct PullResult {
  /// The directory the package was pulled to.
  pub base_dir: PathBuf,
  /// The root file path for the package.
  pub root_path: PathBuf,
}

/// Pull a Wick package from a registry.
pub async fn pull(
  reference: &str,
  username: Option<&str>,
  password: Option<&str>,
  insecure: Option<bool>,
) -> Result<PullResult, Error> {
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

  let download_dir = match create_directory_structure(reference).await {
    Ok(path) => {
      println!("Directory created successfully: {}", path.display());
      path
    }
    Err(e) => return Err(Error::DirectoryCreationFailed(e.to_string())),
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
  Ok(PullResult {
    base_dir: download_dir,
    root_path: PathBuf::from(root_file),
  })
}
