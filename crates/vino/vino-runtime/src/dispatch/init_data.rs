use std::collections::HashMap;

use crate::dev::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct InitData {
  pub seed: u64,
}

impl Default for InitData {
  fn default() -> Self {
    Self { seed: new_seed() }
  }
}

impl From<HashMap<String, String>> for InitData {
  fn from(map: HashMap<String, String>) -> Self {
    let mut data = InitData::default();
    for (k, v) in map {
      if k == "seed" {
        data.seed = v.parse::<u64>().unwrap_or_default();
      }
    }
    data
  }
}
