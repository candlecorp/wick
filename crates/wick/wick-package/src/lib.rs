mod media_types;
mod annotations;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs};

use oci_distribution::client::{Client, ClientConfig, ImageLayer};
use oci_distribution::manifest::{OciDescriptor, OciImageManifest};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use sha256::digest;

// WickFile struct
#[derive(Debug)]
pub struct WickFile {
  path: PathBuf,
  hash: String,
  media_type: String,
  contents: Vec<u8>,
}

impl WickFile {
  fn from_path(path: &Path) -> Self {
    let file_string = fs::read_to_string(path).unwrap();
    let hash = format!("sha256:{}", digest(file_string.to_owned()));

    let media_type = "tbd"; // TODO: get media type from api. wick.kind(&path);

    Self {
      path: path.to_path_buf(),
      hash: hash,
      media_type: media_type.to_string(),
      contents: file_string.as_bytes().to_vec(),
    }
  }
}

// WickPackage struct
#[derive(Debug)]
pub struct WickPackage {
  name: String,
  version: String,
  files: Vec<WickFile>,
  annotations: HashMap<String, String>,
}

impl WickPackage {
  pub fn from_directory(path: &Path) -> Self {
    if !path.is_dir() {
      eprintln!("Error: '{}' is not a directory.", path.display());
      std::process::exit(1);
    }

    let files = [
      "./component.yaml",
      "./build/test.signed.wasm",
      "./icon.png",
      "./test/test1.yaml",
    ]; //assets::get_refs(file);

    let mut wick_files: Vec<WickFile> = Vec::new();
    let mut name = "";
    let mut version = "";
    let mut annotations: HashMap<String, String> = HashMap::new();
    let mut parent_dir= PathBuf::new();

    for file in files.iter() {
      let rel_path = PathBuf::from(file);
      let wick_file = WickFile::from_path(&rel_path);
      wick_files.push(wick_file);
      match wick_file.media_type.as_str() {
        assets::type::COMPONENT || assets::type::APPLICATION => { //need assets api
          let metadata = assets::get_metadata(file); //need assets api
          name = &metadata.name;
          version = &metadata.version;
          annotations = metadata.annotations;

          
          let abs_path = get_full_path(&rel_path).unwrap();
          println!("Full path: {}", abs_path.display());
          parent_dir = get_base_path(&abs_path).unwrap();
          println!("parent_dir: {}", parent_dir.display());
        }
        _ => {}
      }
    }

    //check all files to make sure they are in parent_dir
    for file in files.iter() {
      let rel_path = PathBuf::from(file);
      ensure_relative_path(&parent_dir, &rel_path).unwrap();
    }

    Self {
      name: name.to_string(),
      version: version.to_string(),
      files: wick_files,
      annotations: annotations,
    }
  }

  pub fn list_files(&self) -> Vec<&WickFile> {
    self.files.iter().collect()
  }

  pub async fn push(&self, reference: &str, username: Option<&str>, password: Option<&str>) -> &str {
    let image_config_contents = "{}"; //this is the config file for the oci image

    let image_config = oci_distribution::client::Config {
      data: image_config_contents.as_bytes().to_vec(),
      media_type: media_types::CONFIG.to_string(),
      annotations: None,
    };

    let mut image_layer_descriptors: Vec<OciDescriptor> = Vec::new();
    let mut image_layers: Vec<ImageLayer> = Vec::new();

    for file in self.files.iter() {
      let mut annotations_map: HashMap<String, String> = HashMap::new();

      annotations_map.insert(annotations::TITLE.to_string(), file.path.display().to_string());

      let image_layer = ImageLayer {
        data: file.contents,
        media_type: file.media_type,
        annotations: None,
      };

      let image_layer_descriptor = OciDescriptor {
        media_type: image_layer.media_type,
        digest: file.hash,
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
      image_annotations.insert(new_key.to_string(), value.to_string());
    }

    let image_manifest = OciImageManifest {
      schema_version: 2,
      config: OciDescriptor {
        media_type: image_config.media_type,
        digest: format!("sha256:{}", digest(image_config_contents.to_string())),
        size: image_config.data.len() as i64,
        annotations: None,
        urls: None,
      },
      layers: image_layer_descriptors,
      media_type: Some(media_types::MANIFEST.to_string()),
      annotations: Some(image_annotations),
    };

    let client_config = ClientConfig {
      protocol: oci_distribution::client::ClientProtocol::Https,
      ..Default::default()
    };

    let auth = match (username, password) {
      (username, password) => RegistryAuth::Basic(username.to_string(), password.to_string()),
      _ => {
          println!("Both username and password must be supplied. Falling back to anonymous auth");
          RegistryAuth::Anonymous
      }
    };

    let client = Client::new(client_config);
    let image_ref = Reference::from_str(reference)
        .expect("Failed to parse image reference");

    println!("Pushing image to registry: {}", image_ref);  
    
    let result = client
    .push(&image_ref, &image_layers, image_config, &auth, Some(image_manifest)).await;

    match result {
      Ok(push_response) => {
          println!("Image successfully pushed to the registry.");
          println!("Config URL: {}", push_response.config_url);
          println!("Manifest URL: {}", push_response.manifest_url);
          &push_response.manifest_url
      }
      Err(e) => {
          eprintln!("Failed to push the image: {}", e);
          std::process::exit(1);
      }
    }
  }
}


fn get_base_path(full_path: &PathBuf) -> Result<PathBuf, &'static str> {
  let base_path = full_path.parent().ok_or("Error getting base path")?;
  Ok(base_path.to_path_buf())
}

fn get_full_path(rel_path: &PathBuf) -> Result<PathBuf, &'static str> {
  let abs_path = std::fs::canonicalize(rel_path).map_err(|_| "Error getting full path")?;
  Ok(abs_path)
}

fn ensure_relative_path(base_dir: &PathBuf, path: &PathBuf) -> Result<PathBuf, &'static str> {
  // Get the prefix of the path that matches the base directory
  let prefix = path.strip_prefix(base_dir).map_err(|_| "Path is not relative to the given directory")?;

  // Check if the prefix is empty, indicating that the path is not going above the base directory
  if prefix.as_os_str().is_empty() {
      // The path is a valid relative path inside the base directory
      Ok(prefix.to_path_buf())
  } else {
      // The path is going above the base directory
      Err("Path is going above the given directory")
  }
}