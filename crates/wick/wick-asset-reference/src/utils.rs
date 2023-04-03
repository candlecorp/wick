use std::str::FromStr;

use url::Url;

use crate::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn path_to_url(path: &std::path::Path, base: Option<Url>) -> Result<Url> {
  let pathstr = path.to_string_lossy().to_string();
  str_to_url(&pathstr, base)
}

pub fn str_to_url(path: &str, base: Option<Url>) -> Result<Url> {
  let url = match base {
    Some(full_url) => {
      if !full_url.path().ends_with('/') {
        let mut url = full_url.clone();
        url.set_path(&format!("{}/", full_url.path()));
        url.join(path)?
      } else {
        full_url.join(path)?
      }
    }
    None => match Url::from_str(path) {
      Ok(url) => url,
      Err(e) => match e {
        url::ParseError::RelativeUrlWithoutBase => {
          let mut cwd = std::env::current_dir().unwrap();
          cwd.push(path);
          Url::from_file_path(cwd).map_err(|_| Error::BadUrl(path.to_owned()))?
        }
        e => return Err(e.into()),
      },
    },
  };
  Ok(url)
}
