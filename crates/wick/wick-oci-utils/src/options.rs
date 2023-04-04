use std::path::PathBuf;

use oci_distribution::secrets::RegistryAuth;

#[derive(Default)]
#[must_use]
pub struct OciOptions {
  pub(crate) allow_latest: bool,
  pub(crate) allow_insecure: Vec<String>,
  pub(crate) username: Option<String>,
  pub(crate) password: Option<String>,
  pub(crate) base_dir: Option<PathBuf>,
  pub(crate) cache_dir: Option<PathBuf>,
  pub(crate) overwrite: bool,
}

impl OciOptions {
  pub fn allow_latest(mut self, allow_latest: bool) -> Self {
    self.allow_latest = allow_latest;
    self
  }

  pub fn allow_insecure(mut self, allow_insecure: Vec<String>) -> Self {
    self.allow_insecure = allow_insecure;
    self
  }

  pub fn cache_dir(mut self, cache_dir: Option<PathBuf>) -> Self {
    self.cache_dir = cache_dir;
    self
  }

  pub fn base_dir(mut self, base_dir: Option<PathBuf>) -> Self {
    self.base_dir = base_dir;
    self
  }

  pub fn username(mut self, username: Option<String>) -> Self {
    self.username = username;
    self
  }

  pub fn password(mut self, password: Option<String>) -> Self {
    self.password = password;
    self
  }

  pub fn overwrite(mut self, overwrite: bool) -> Self {
    self.overwrite = overwrite;
    self
  }

  #[must_use]
  pub fn get_cache_dir(&self) -> Option<&PathBuf> {
    self.cache_dir.as_ref()
  }

  #[must_use]
  pub fn get_base_dir(&self) -> Option<&PathBuf> {
    self.base_dir.as_ref()
  }

  #[must_use]
  pub fn get_auth(&self) -> RegistryAuth {
    match (self.username.as_ref(), self.password.as_ref()) {
      (Some(username), Some(password)) => RegistryAuth::Basic(username.clone(), password.clone()),
      _ => RegistryAuth::Anonymous,
    }
  }
}

impl std::fmt::Debug for OciOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OciOptions")
      .field("allow_latest", &self.allow_latest)
      .field("allow_insecure", &self.allow_insecure)
      .field("username", &self.username)
      .field("password", &self.password.as_ref().map(|_| "********"))
      .finish()
  }
}
