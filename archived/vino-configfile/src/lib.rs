use std::path::PathBuf;

use directories_next::BaseDirs;
use serde::Deserialize;
#[derive(Debug, Deserialize, Default)]
pub struct VinoConfig {
  pub cache_dir: Option<String>,
}

#[allow(dead_code)]
fn load_configfile<T: AsRef<str>>(file: T) -> VinoConfig {
  let mut dir = match BaseDirs::new() {
    Some(base_dirs) => base_dirs.home_dir().to_path_buf(),
    None => PathBuf::new(),
  };

  dir.push(file.as_ref());
  if let Ok(result) = std::fs::read_to_string(dir) {
    let config: VinoConfig = toml::from_str(&result).unwrap_or_default();

    println!("{:?}", config);
    config
  } else {
    VinoConfig::default()
  }
}

#[cfg(test)]
mod tests {
  use crate::load_configfile;
  fn init() {
    env_logger::init();
  }
  #[test]
  fn runs_crud_api_config() {
    init();
    let config = load_configfile(".vinocfg");
    println!("{:?}", config);
  }
}
