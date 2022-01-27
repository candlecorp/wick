use serde::{
  de::{IgnoredAny, SeqAccess, Visitor},
  Deserializer,
};
use wapc::WasiParams;

pub const WASI_CONFIG_PREOPENED_DIRS: &str = "preopened_dirs";
pub const WASI_CONFIG_MAP_DIRS: &str = "map_dirs";
pub const WASI_CONFIG_ARGV: &str = "argv";
pub const WASI_CONFIG_ENV: &str = "env";

pub mod error {
  use std::env::VarError;

  #[derive(thiserror::Error, Debug)]
  pub enum WasiConfigError {
    #[error("WasiParams could not be created from configuration: {0}")]
    EnvLookupFailed(shellexpand::LookupError<VarError>),

    #[error("WasiParams could not be created from configuration: {0}")]
    ConversionFailed(serde_json::Error),
  }

  impl From<serde_json::Error> for WasiConfigError {
    fn from(e: serde_json::Error) -> Self {
      WasiConfigError::ConversionFailed(e)
    }
  }
  impl From<shellexpand::LookupError<VarError>> for WasiConfigError {
    fn from(e: shellexpand::LookupError<VarError>) -> Self {
      WasiConfigError::EnvLookupFailed(e)
    }
  }
}

#[derive(Default, Debug)]
struct StringPair(String, String);

impl<'de> serde::Deserialize<'de> for StringPair {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringPairVisitor;

    impl<'de> Visitor<'de> for StringPairVisitor {
      type Value = StringPair;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a String pair")
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
      where
        V: SeqAccess<'de>,
      {
        let s = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let n = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        // This is very important!
        while let Some(IgnoredAny) = seq.next_element()? {
          // Ignore rest
        }

        Ok(StringPair(s, n))
      }
    }

    deserializer.deserialize_seq(StringPairVisitor)
  }
}

#[derive(Default, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WasiParamsSerde {
  #[serde(rename = "argv", default)]
  argv: Vec<String>,
  #[serde(rename = "map_dirs", default)]
  map_dirs: Vec<StringPair>,
  #[serde(rename = "env", default)]
  env_vars: Vec<StringPair>,
  #[serde(rename = "preopened_dirs", default)]
  preopened_dirs: Vec<String>,
}

/// Extract [WasiParams] from a JSON-like struct. This function only emits warnings on invalid values.
pub fn config_to_wasi(
  cfg: Option<serde_json::Value>,
  wasi: Option<WasiParams>,
) -> Result<WasiParams, self::error::WasiConfigError> {
  let mut wasi = wasi.unwrap_or_default();
  trace!("WASM:WASI: Passed config is {:?}", cfg);
  trace!("WASM:WASI: Passed wasi params are {:?}", wasi);
  if let Some(v) = cfg {
    let wasi_cfg = serde_json::from_value::<WasiParamsSerde>(v)?;
    trace!("WASM:WASI: Config is {:?}", wasi_cfg);
    for dir in wasi_cfg.preopened_dirs {
      wasi.preopened_dirs.push(shellexpand::env(&dir)?.into());
    }
    for map in wasi_cfg.map_dirs {
      wasi
        .map_dirs
        .push((shellexpand::env(&map.0)?.into(), shellexpand::env(&map.1)?.into()));
    }
    for env in wasi_cfg.env_vars {
      wasi
        .env_vars
        .push((shellexpand::env(&env.0)?.into(), shellexpand::env(&env.1)?.into()));
    }
    for argv in wasi_cfg.argv {
      wasi.argv.push(shellexpand::env(&argv)?.into());
    }
  } else {
    debug!("WASM: No config present, using default WASI configuration.");
  }
  Ok(wasi)
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test_logger::test]
  fn full_config_to_wasiparams() -> anyhow::Result<()> {
    let cfg = json!({
      "preopened_dirs": ["/foo", "/bar"],
      "map_dirs": [
        ["a", "b"]
      ],
      "env": [["FOO", "BAR"]],
      "argv": ["HEY", "YOU"]
    });
    let actual = config_to_wasi(Some(cfg), None)?;
    let expected = WasiParams {
      preopened_dirs: vec!["/foo".to_owned(), "/bar".to_owned()],
      map_dirs: vec![("a".to_owned(), "b".to_owned())],
      env_vars: vec![("FOO".to_owned(), "BAR".to_owned())],
      argv: vec!["HEY".to_owned(), "YOU".to_owned()],
    };
    // TODO: remove the field comparison when WasiParams implements Eq
    assert_eq!(actual.preopened_dirs, expected.preopened_dirs);
    assert_eq!(actual.map_dirs, expected.map_dirs);
    assert_eq!(actual.env_vars, expected.env_vars);
    assert_eq!(actual.argv, expected.argv);
    Ok(())
  }

  #[test_logger::test]
  fn partial_config_stringlist() -> anyhow::Result<()> {
    let cfg = json!({
      "preopened_dirs": ["/foo", "/bar"],
    });
    let actual = config_to_wasi(Some(cfg), None)?;
    let expected = WasiParams {
      preopened_dirs: vec!["/foo".to_owned(), "/bar".to_owned()],
      ..Default::default()
    };
    // TODO: remove the field comparison when WasiParams implements Eq
    assert_eq!(actual.preopened_dirs, expected.preopened_dirs);
    assert_eq!(actual.map_dirs, expected.map_dirs);
    assert_eq!(actual.env_vars, expected.env_vars);
    assert_eq!(actual.argv, expected.argv);
    Ok(())
  }

  #[test_logger::test]
  fn partial_config_stringpair() -> anyhow::Result<()> {
    let cfg = json!({
      "map_dirs": [
        ["a", "b"]
      ]
    });
    let actual = config_to_wasi(Some(cfg), None)?;
    let expected = WasiParams {
      map_dirs: vec![("a".to_owned(), "b".to_owned())],
      ..Default::default()
    };
    // TODO: remove the field comparison when WasiParams implements Eq
    assert_eq!(actual.preopened_dirs, expected.preopened_dirs);
    assert_eq!(actual.map_dirs, expected.map_dirs);
    assert_eq!(actual.env_vars, expected.env_vars);
    assert_eq!(actual.argv, expected.argv);
    Ok(())
  }

  #[test_logger::test]
  fn mixed_config() -> anyhow::Result<()> {
    let cfg = json!({
      "invalid wasi value": true,
      "map_dirs": [
        ["a", "b"]
      ]
    });
    let result = config_to_wasi(Some(cfg), None);
    assert!(result.is_err());
    Ok(())
  }
}
