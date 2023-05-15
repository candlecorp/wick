use std::path::PathBuf;
use std::vec;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::Error;

#[derive(Debug, Default, Deserialize, Serialize)]
#[allow(unused)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Settings {
  #[serde(default)]
  /// Logging configuration.
  pub logging: Logging,
  #[serde(default)]
  /// Registry credentials.
  pub credentials: Vec<Credential>,
  #[serde(skip)]
  /// Where this configuration was loaded from.
  pub source: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
/// Logging configuration.
pub struct Logging {
  /// Logging level.
  #[serde(default)]
  pub level: LogLevel,
  /// Logging modifier.
  #[serde(default)]
  pub modifier: LogModifier,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
/// Registry credentials.
pub struct Credential {
  /// Registry URL.
  pub scope: String,
  /// Authentication method.
  pub auth: Auth,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
/// Authentication methods.
pub enum Auth {
  /// Basic authentication.
  Basic(BasicAuth),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
/// Basic authentication.
pub struct BasicAuth {
  /// Username.
  pub username: String,
  /// Password.
  pub password: String,
}

impl std::fmt::Debug for BasicAuth {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BasicAuth")
      .field("username", &self.username)
      .field("password", &"<HIDDEN>")
      .finish()
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Logging levels.
pub enum LogLevel {
  /// Disable loging
  Off,
  /// Errors only.
  Error,
  /// Warnings and errors only.
  Warn,
  /// Info-level logging.
  Info,
  /// Debug logging.
  Debug,
  /// Trace logging.
  Trace,
}

impl Default for LogLevel {
  fn default() -> Self {
    Self::Info
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Logging modifiers.
pub enum LogModifier {
  /// No modifiers.
  None,
  /// Additional location and file data.
  Verbose,
  /// Include all logging possible.
  Silly,
}

impl Default for LogModifier {
  fn default() -> Self {
    Self::None
  }
}

impl Settings {
  pub fn new() -> Self {
    let extensions = vec!["yaml", "yml"];

    let mut config_locations = Vec::new();
    if let Some(path) = wick_xdg::Files::UserConfigFile.path() {
      config_locations.push(path.to_string_lossy().to_string());
    }
    config_locations.push(wick_xdg::Files::CONFIG_FILE_NAME.to_owned());

    tracing::debug!(
      paths = ?config_locations.iter().map(|v| format!("{}.({})",v,extensions.join(","))).collect::<Vec<_>>(),
      "wick:settings: searching for config files"
    );

    let mut files = Vec::new();
    for path in config_locations {
      let path = PathBuf::from(path);
      for ext in &extensions {
        let mut path = path.clone();
        path.set_extension(ext);
        if path.exists() {
          match std::fs::read_to_string(&path) {
            Ok(src) => match serde_yaml::from_str::<Settings>(&src) {
              Ok(mut settings) => {
                debug!(file=%path.display(),"wick:settings: found config");
                settings.source = Some(path.clone());
                files.push(settings);

                break; // only load the first one, fix when merging is implemented.
              }
              Err(e) => {
                warn!(error=%e,file=%path.display(),"wick:settings: failed to parse config");
              }
            },
            Err(e) => {
              warn!(error=%e,file=%path.display(),"wick:settings: failed to read config");
            }
          };
        }
      }
    }

    debug!("wick:settings: loaded");

    // You can deserialize (and thus freeze) the entire configuration as
    if !files.is_empty() {
      files.remove(0)
    } else {
      Self::default()
    }
  }

  pub fn save(&self) -> Result<(), Error> {
    let source = self.source.as_ref().ok_or(Error::NoSource)?;
    let yaml = serde_yaml::to_string(self).unwrap();
    std::fs::write(source, yaml).map_err(|e| Error::SaveFailed(source.clone(), e))?;
    Ok(())
  }
}
