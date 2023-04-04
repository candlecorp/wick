use std::path::PathBuf;

use oci_distribution::client::ClientConfig;
use oci_distribution::Client;

use super::{annotations, media_types};
use crate::utils::{create_directory_structure, get_cache_directory};
use crate::{AssetManifest, Error, OciOptions};

/// Result of a pull operation.
#[derive(Debug, Clone)]
pub struct PullResult {
  /// The directory the package was pulled to.
  pub base_dir: PathBuf,
  /// The root file path for the package.
  pub root_path: PathBuf,
}

/// Pull a Wick package from a registry.
pub async fn pull(reference: &str, options: &OciOptions) -> Result<PullResult, Error> {
  let (image_ref, protocol) = crate::utils::parse_reference_and_protocol(reference, &options.allow_insecure)?;

  let cache_dir = get_cache_directory(reference, options.get_cache_dir().cloned())?;
  let download_dir = options.get_base_dir().map_or_else(|| cache_dir, |v| v.clone());

  let manifest_file = download_dir.join(AssetManifest::FILENAME);
  if manifest_file.exists() {
    debug!(cache_hit = true, "remote asset");
    let json = tokio::fs::read_to_string(&manifest_file).await?;
    let manifest: AssetManifest = serde_json::from_str(&json).map_err(|_| Error::InvalidManifest(manifest_file))?;
    return Ok(PullResult {
      base_dir: download_dir,
      root_path: manifest.root,
    });
  }
  debug!(cache_hit = false, "remote asset");

  let client_config = ClientConfig {
    protocol,
    ..Default::default()
  };

  let mut client = Client::new(client_config);
  let auth = options.get_auth();

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
      debug!("Image successfully pulled from the registry.");
      pull_response
    }
    Err(e) => {
      return Err(Error::PullFailed(e.to_string()));
    }
  };

  let download_dir = create_directory_structure(download_dir).await?;

  let mut root_file: Option<String> = None;

  let mut would_overwrite: Vec<PathBuf> = Vec::new();
  for layer in &image_data.layers {
    let layer_title = layer
      .annotations
      .as_ref()
      .and_then(|v| v.get(annotations::TITLE).cloned())
      .ok_or(Error::NoTitle)?;
    let layer_path = download_dir.join(&layer_title);

    // If canonicalize succeeds, the path exists and we would overwrite it.
    if let Ok(path) = layer_path.canonicalize() {
      would_overwrite.push(path);
    }
  }
  if !would_overwrite.is_empty() && !options.overwrite {
    return Err(Error::WouldOverwrite(would_overwrite));
  }

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
  let manifest = AssetManifest::new(PathBuf::from(&root_file));
  let contents = serde_json::to_string(&manifest).unwrap();
  tokio::fs::write(download_dir.join(AssetManifest::FILENAME), contents).await?;

  debug!(path = root_file, "Root file");
  Ok(PullResult {
    base_dir: download_dir,
    root_path: PathBuf::from(root_file),
  })
}
