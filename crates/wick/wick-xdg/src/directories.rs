#![allow(clippy::option_if_let_else)]
use std::path::PathBuf;

use crate::error::Error;
use crate::PROJECT_ID;

#[derive(Debug, Clone, Copy)]
pub enum Directories {
  /// Directories to store information related to the state of Wick.
  GlobalState,
  /// Directories to store caches.
  RelativeCache,
  /// Directories to store global caches.
  GlobalCache,
}

impl Directories {
  pub fn basedir(&self) -> Result<PathBuf, Error> {
    match self {
      Self::GlobalState => global_state_dir(),
      Self::RelativeCache => Ok(Cache::Root.basedir()),
      Self::GlobalCache => Ok(global_cache_dir()?.join(Cache::Root.basedir())),
    }
  }
}

const CACHE_BASE: &str = ".wick";

#[derive(Debug, Clone, Copy)]
/// The type of cache to use.
pub enum Cache {
  /// The root cache.
  Root,
  /// The component cache directory.
  Assets,
}

impl Cache {
  /// Get the path to the cache.
  #[must_use]
  pub fn basedir(&self) -> PathBuf {
    PathBuf::from(CACHE_BASE).join("remote")
  }
}

fn global_state_dir() -> Result<PathBuf, Error> {
  #[cfg(not(target_os = "windows"))]
  return Ok(match xdg::BaseDirectories::with_prefix(PROJECT_ID) {
    Ok(xdg) => xdg.get_state_home(),
    Err(_) => std::env::current_dir().map_err(|_| Error::Pwd)?,
  });
  #[cfg(target_os = "windows")]
  return Ok(match std::env::var("LOCALAPPDATA") {
    Ok(var) => PathBuf::from(var).join(PROJECT_ID),
    Err(_) => std::env::current_dir().map_err(|_| Error::Pwd)?,
  });
}

fn global_cache_dir() -> Result<PathBuf, Error> {
  #[cfg(not(target_os = "windows"))]
  return Ok(match xdg::BaseDirectories::with_prefix(PROJECT_ID) {
    Ok(xdg) => xdg.get_cache_home(),
    Err(_) => std::env::temp_dir(),
  });
  #[cfg(target_os = "windows")]
  return Ok(std::env::temp_dir());
}
