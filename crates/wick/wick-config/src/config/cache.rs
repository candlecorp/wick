use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use wick_asset_reference::AssetReference;

use super::UninitializedConfiguration;

pub(super) static CONFIG_CACHE: Lazy<Mutex<HashMap<AssetReference, UninitializedConfiguration>>> =
  Lazy::new(|| Mutex::new(HashMap::new()));
