use std::path::PathBuf;

pub(crate) static DOTROOT: &str = ".wick";
pub(crate) static CACHE: &str = "cache";
pub(crate) static DATA: &str = "data";
pub(crate) static CONFIG: &str = "config";
pub(crate) static STAGING: &str = "staging";

pub(crate) fn relative_root() -> PathBuf {
  PathBuf::from(DOTROOT)
}

pub(crate) fn global_root() -> PathBuf {
  home_dir().unwrap_or_else(pwd).join(DOTROOT)
}

pub(crate) fn global_data_dir() -> PathBuf {
  global_root().join(DATA)
}

pub(crate) fn user_config_dir() -> PathBuf {
  global_root().join(CONFIG)
}

#[allow(clippy::option_if_let_else)]
fn home_dir() -> Option<PathBuf> {
  #[cfg(not(target_os = "windows"))]
  return match std::env::var("HOME") {
    Ok(dir) => Some(PathBuf::from(dir)),
    Err(_) => None,
  };
  #[cfg(target_os = "windows")]
  return match std::env::var("USERPROFILE") {
    Ok(dir) => Some(PathBuf::from(dir)),
    Err(_) => None,
  };
}

#[allow(clippy::expect_used)]
fn pwd() -> PathBuf {
  std::env::current_dir().expect("could not get home dir or pwd, can not continue")
}
