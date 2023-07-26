use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use nkeys::{KeyPair, KeyPairType};

use crate::io::{mkdirp, read_to_string, write_bytes};

static KEY_EXT: &str = ".nk";

#[derive(Debug, Clone, Args)]
pub(crate) struct GenerateCommon {
  /// Location of key files. Defaults to <CONFIG_HOME>/wick/config/keys).
  #[clap(long = "directory", env = "WICK_KEYS", action)]
  pub(crate) directory: Option<PathBuf>,

  /// The account key to sign with.
  #[clap(long = "signer-key", env = "WICK_SIGNER_KEY", action, hide_env_values = true)]
  pub(crate) signer: Option<String>,

  /// The subject key to sign with.
  #[clap(long = "subject-key", env = "WICK_SUBJECT_KEY", action, hide_env_values = true)]
  pub(crate) subject: Option<String>,

  /// Set the token expiration in days. By default the token will never expire.
  #[clap(short = 'x', long = "expires", action)]
  pub(crate) expires_in_days: Option<u64>,

  /// Period in days before token becomes valid. By default the token will be valid immediately.
  #[clap(long, action)]
  pub(crate) wait: Option<u64>,
}

pub(crate) fn get_key_files(directory: Option<PathBuf>) -> Result<(PathBuf, Vec<String>)> {
  let dir = directory.map_or_else(get_key_home, |dir| dir);

  let mut keys = vec![];
  let entries = fs::read_dir(dir.clone()).map_err(|e| anyhow!("Could not open {}: {} ", dir.to_string_lossy(), e))?;

  for entry in entries.flatten() {
    if let Some(file) = entry.file_name().to_str() {
      if file.ends_with(KEY_EXT) {
        keys.push(file.to_owned());
      }
    }
  }

  Ok((dir, keys))
}

pub(crate) async fn get_key(directory: Option<PathBuf>, path: PathBuf) -> Result<KeyPair> {
  let mut filepath = directory.map_or_else(get_key_home, |dir| dir);

  filepath.push(path);
  let seed = read_to_string(filepath).await?;
  let kp = KeyPair::from_seed(&seed)?;

  Ok(kp)
}

fn get_key_home() -> PathBuf {
  let env = wick_xdg::Settings::new();
  env.config_dir().join("keys")
}

pub(crate) async fn get_module_keys(
  name: Option<&str>,
  directory: Option<PathBuf>,
  signer_key: Option<String>,
  subject_key: Option<String>,
) -> Result<(KeyPair, KeyPair)> {
  let account_keys = match signer_key {
    Some(seed) => KeyPair::from_seed(&seed)?,
    None => get_or_create(name, directory.clone(), KeyPairType::Account).await?,
  };

  let subject_keys = match subject_key {
    Some(seed) => KeyPair::from_seed(&seed)?,
    None => get_or_create(name, directory, KeyPairType::Module).await?,
  };

  Ok((account_keys, subject_keys))
}

pub(crate) async fn get_or_create(
  name: Option<&str>,
  directory: Option<PathBuf>,
  kp_type: KeyPairType,
) -> Result<KeyPair> {
  if name.is_none() {
    return Err(anyhow!("Component name must be supplied to generate signing keys."));
  }
  let name = name.unwrap();

  let dir = directory.map_or_else(get_key_home, |dir| dir);

  let module_name = match kp_type {
    KeyPairType::Account => std::env::var("USER").unwrap_or_else(|_| "user".to_owned()),
    _ => PathBuf::from(name).file_stem().unwrap().to_str().unwrap().to_owned(),
  };

  let path = format!(
    "{}/{}_{}{}",
    dir.to_string_lossy(),
    module_name,
    keypair_type_to_string(&kp_type),
    KEY_EXT
  );

  let seed = match read_to_string(&path).await {
    Ok(seed) => seed,
    Err(_e) => {
      info!("No keypair found at \"{}\". Generating new keypair.", path);

      let kp = KeyPair::new(kp_type);
      let seed = kp.seed()?;
      let path = Path::new(&path);
      let parent = path.parent().unwrap();
      mkdirp(parent).await?;
      write_bytes(path, seed.as_bytes()).await?;
      seed
    }
  };

  Ok(KeyPair::from_seed(&seed)?)
}

pub(crate) fn keypair_type_to_string(keypair_type: &KeyPairType) -> String {
  use KeyPairType::*;
  match keypair_type {
    Account => "account".to_owned(),
    Cluster => "cluster".to_owned(),
    Service => "service".to_owned(),
    Module => "module".to_owned(),
    Server => "server".to_owned(),
    Operator => "operator".to_owned(),
    User => "user".to_owned(),
  }
}
