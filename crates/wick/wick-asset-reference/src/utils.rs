use std::path::PathBuf;

use normpath::PathExt;
use tracing::trace;

use crate::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn path_to_url(path: &std::path::Path, base: Option<String>) -> Result<String> {
  let pathstr = path.to_string_lossy().to_string();
  str_to_url(&pathstr, base)
}

#[allow(clippy::option_if_let_else)]
pub fn str_to_url(path: &str, base: Option<String>) -> Result<String> {
  let url = match base {
    Some(full_url) => {
      trace!("Resolving path to baseurl: {} + {}", full_url, path);
      let p = PathBuf::from(&full_url).join(path);
      p.normalize()
        .map_err(|e| Error::BaseUrlFailure(full_url, e.to_string()))?
        .as_path()
        .to_string_lossy()
        .to_string()
    }
    None => {
      if !path.starts_with(std::path::MAIN_SEPARATOR) {
        trace!("Path is relative, converting to absolute path: {}", path);
        let absolute = std::env::current_dir().unwrap().join(path);
        absolute.display().to_string()
      } else {
        trace!("Path is absolute, leaving alone: {}", path);

        path.to_owned()
      }
    }
  };
  Ok(url)
}
