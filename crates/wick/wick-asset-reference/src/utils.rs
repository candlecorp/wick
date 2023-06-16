use std::path::PathBuf;

use normpath::PathExt;

use crate::Error;

type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::option_if_let_else)]
pub fn normalize_path(path: &std::path::Path, base: Option<PathBuf>) -> Result<PathBuf> {
  let url = match base {
    Some(full_url) => {
      let p = PathBuf::from(&full_url).join(path);
      p.normalize()
        .map_err(|e| Error::NormalizationFailure(p.to_string_lossy().to_string(), e.to_string()))?
        .as_path()
        .to_owned()
    }
    None => {
      if !path.is_absolute() {
        std::env::current_dir().unwrap().join(path)
      } else {
        PathBuf::from(path)
      }
    }
  };
  Ok(url)
}
