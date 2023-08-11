use std::path::PathBuf;

use oci_distribution::secrets::RegistryAuth;

#[derive(Clone, Debug, Copy, serde::Serialize)]
pub enum OnExisting {
  Ignore,
  Overwrite,
  Error,
}

#[derive(getset::Getters, getset::Setters, Clone, serde::Serialize)]
#[must_use]
pub struct OciOptions {
  #[getset(get = "pub", set = "pub")]
  pub(crate) allow_latest: bool,
  #[getset(get = "pub", set = "pub")]
  pub(crate) allow_insecure: Vec<String>,
  #[getset(get = "pub", set = "pub")]
  pub(crate) username: Option<String>,
  #[getset(get = "pub", set = "pub")]
  pub(crate) password: Option<String>,
  #[getset(get = "pub", set = "pub")]
  pub(crate) cache_dir: PathBuf,
  #[getset(get = "pub", set = "pub")]
  pub(crate) on_existing: OnExisting,
}

impl Default for OciOptions {
  fn default() -> Self {
    let xdg = wick_xdg::Settings::new();
    Self {
      allow_latest: false,
      allow_insecure: vec![],
      username: None,
      password: None,
      cache_dir: xdg.global().cache().clone(),
      on_existing: OnExisting::Ignore,
    }
  }
}

impl OciOptions {
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
