use std::path::{Path, PathBuf};
use std::sync::Arc;

use asset_container::{Asset, AssetManager};
use parking_lot::RwLock;
use tracing::error;
use wick_asset_reference::AssetReference;

#[derive(Debug, Clone, Default)]

pub struct Glob {
  pub(crate) glob: String,
  pub(crate) assets: Arc<RwLock<Vec<AssetReference>>>,
  pub(crate) baseurl: Arc<RwLock<Option<PathBuf>>>,
}

impl Glob {
  pub fn new(glob: impl AsRef<str>) -> Self {
    Self {
      glob: glob.as_ref().to_owned(),
      ..Default::default()
    }
  }
}

impl AssetManager for Glob {
  type Asset = AssetReference;

  fn set_baseurl(&self, baseurl: &Path) {
    *self.baseurl.write() = Some(baseurl.to_owned());

    let assets = self.assets();
    for asset in assets.iter() {
      asset.update_baseurl(baseurl);
    }
  }

  fn assets(&self) -> asset_container::Assets<Self::Asset> {
    let root = self.baseurl.read().as_ref().map(PathBuf::from).unwrap_or_default();
    let pattern = root.join(&self.glob);
    let entries = match glob::glob(pattern.to_str().unwrap()) {
      Ok(entries) => entries,
      Err(e) => {
        error!("Failed to glob: {}", e);
        panic!();
      }
    };
    let mut asset_refs = Vec::new();
    for entry in entries {
      asset_refs.push(AssetReference::new(&entry.unwrap().to_string_lossy()));
    }

    *self.assets.write() = asset_refs.clone();

    let mut assets: asset_container::Assets<Self::Asset> = asset_container::Assets::new(vec![], self.get_asset_flags());

    for asset in asset_refs {
      assets.push_owned(asset);
    }

    assets
  }
}
