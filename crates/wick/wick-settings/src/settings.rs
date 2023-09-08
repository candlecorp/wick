use std::path::PathBuf;
use std::vec;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::Error;

#[derive(Debug, Default, Deserialize, Serialize, derive_builder::Builder)]
#[allow(unused)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[builder(pattern = "owned", default)]
pub struct Settings {
  #[serde(default)]
  /// Logging configuration.
  pub trace: TraceSettings,
  /// Registry credentials.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub credentials: Vec<Credential>,
  #[serde(skip)]
  /// Where this configuration was loaded from.
  pub source: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
/// Logging configuration.
pub struct TraceSettings {
  /// OTLP endpoint endpoint.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub otlp: Option<String>,
  /// Logging level.
  #[serde(default)]
  pub level: LogLevel,
  /// Logging modifier.
  #[serde(default)]
  pub modifier: LogModifier,
  /// Telemetry filter settings.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub telemetry: Option<LogSettings>,
  /// STDERR logging settings.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stderr: Option<LogSettings>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
/// Log filter settings
pub struct LogSettings {
  /// Log event filter.
  pub filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
/// Registry credentials.
pub struct Credential {
  /// Registry URL.
  pub scope: String,
  /// Authentication method.
  pub auth: Auth,
}

impl Credential {
  /// Create a new credential entry.
  #[must_use]
  pub const fn new(scope: String, auth: Auth) -> Self {
    Self { scope, auth }
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_enums)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
/// Authentication methods.
pub enum Auth {
  /// Basic authentication.
  Basic(BasicAuth),
}

#[derive(Deserialize, Serialize)]
#[non_exhaustive]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
/// Basic authentication.
pub struct BasicAuth {
  /// Username.
  pub username: String,
  /// Password.
  pub password: String,
}

impl BasicAuth {
  /// Create a new basic authentication entry.
  #[must_use]
  pub const fn new(username: String, password: String) -> Self {
    Self { username, password }
  }
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
#[allow(clippy::exhaustive_enums)]
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
#[allow(clippy::exhaustive_enums)]
#[serde(rename_all = "snake_case")]
/// Logging modifiers.
pub enum LogModifier {
  /// No modifiers.
  None,
  /// Additional location and file data.
  Verbose,
}

impl Default for LogModifier {
  fn default() -> Self {
    Self::None
  }
}

impl Settings {
  pub fn new() -> Self {
    let _span = tracing::info_span!("settings").entered();
    let extensions = vec!["yaml", "yml"];

    let xdg = wick_xdg::Settings::new();

    let config_locations = vec![
      xdg.config_dir().join(xdg.configfile_basename()),
      PathBuf::from(xdg.configfile_basename()),
    ];

    tracing::debug!(
      paths = ?config_locations.iter().map(|v| format!("{}.({})",v.display(),extensions.join(","))).collect::<Vec<_>>(),
      "searching for config files"
    );

    let mut files = find_settings(&config_locations, &extensions);

    debug!("loaded");

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

#[allow(clippy::cognitive_complexity)]
fn find_settings(config_locations: &[PathBuf], extensions: &[&str]) -> Vec<Settings> {
  let mut files = Vec::new();
  for path in config_locations {
    for ext in extensions {
      let mut path = path.clone();
      path.set_extension(ext);
      if path.exists() {
        match std::fs::read_to_string(&path) {
          Ok(src) => match serde_yaml::from_str::<Settings>(&src) {
            Ok(mut settings) => {
              debug!(file=%path.display(),"found config");
              settings.source = Some(path.clone());
              files.push(settings);

              break; // only load the first one, fix when merging is implemented.
            }
            Err(e) => {
              warn!(error=%e,file=%path.display(),"failed to parse config");
            }
          },
          Err(e) => {
            warn!(error=%e,file=%path.display(),"failed to read config");
          }
        };
      }
    }
  }
  files
}
