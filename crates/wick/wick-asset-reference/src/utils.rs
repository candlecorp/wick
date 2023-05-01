use std::path::PathBuf;

use normpath::PathExt;

use crate::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn normalize_path(path: &std::path::Path, base: Option<String>) -> Result<String> {
  let pathstr = path.to_string_lossy().to_string();
  normalize_path_str(&pathstr, base)
}

#[allow(clippy::option_if_let_else)]
pub fn normalize_path_str(path: &str, base: Option<String>) -> Result<String> {
  let url = match base {
    Some(full_url) => {
      let p = PathBuf::from(&full_url).join(path);
      p.normalize()
        .map_err(|e| Error::NormalizationFailure(p.to_string_lossy().to_string(), e.to_string()))?
        .as_path()
        .to_string_lossy()
        .to_string()
    }
    None => {
      if !path.starts_with(std::path::MAIN_SEPARATOR) {
        let absolute = std::env::current_dir().unwrap().join(path);
        absolute.display().to_string()
      } else {
        path.to_owned()
      }
    }
  };
  Ok(url)
}
