#![allow(clippy::option_if_let_else)]
use std::path::PathBuf;

use crate::PROJECT_ID;

#[derive(Debug, Clone, Copy)]
pub enum Files {
  /// The user-specific configuration file location.
  UserConfigFile,
}

impl Files {
  pub const CONFIG_FILE_NAME: &str = "config";
  #[must_use]
  pub fn path(&self) -> Option<PathBuf> {
    match self {
      Self::UserConfigFile => user_config_file(),
    }
  }
}

fn user_config_file() -> Option<PathBuf> {
  #[cfg(not(target_os = "windows"))]
  return match xdg::BaseDirectories::with_prefix(PROJECT_ID) {
    Ok(xdg) => Some(xdg.get_config_home().join(Files::CONFIG_FILE_NAME)),
    Err(_) => None,
  };
  #[cfg(target_os = "windows")]
  return match std::env::var("LOCALAPPDATA") {
    Ok(var) => Some(PathBuf::from(var).join(PROJECT_ID).join(Files::CONFIG_FILE_NAME)),
    Err(_) => None,
  };
}
