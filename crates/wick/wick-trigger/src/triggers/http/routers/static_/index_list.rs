use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};

use hyper::{Body, Request, Response, StatusCode};
use once_cell::sync::Lazy;

static INDEX_TEMPLATE: Lazy<liquid::Template> = Lazy::new(|| {
  let template = include_str!("template.html");
  liquid::ParserBuilder::with_stdlib()
    .build()
    .unwrap()
    .parse(template)
    .unwrap()
});

#[derive(serde::Serialize, Debug)]
struct Entry {
  name: String,
  is_dir: bool,
}

pub(super) enum StaticError {
  Template,
  Io(std::io::Error),
}

impl From<std::io::Error> for StaticError {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl std::fmt::Display for StaticError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Template => write!(f, "Template error"),
      Self::Io(e) => e.fmt(f),
    }
  }
}

pub(super) async fn create<B>(root: &Path, request: &Request<B>) -> Result<Response<Body>, StaticError>
where
  B: Send + Sync + 'static,
{
  let resolved = RequestedPath::resolve(request.uri().path());
  let mut base = root.to_path_buf();
  base.push(&resolved.sanitized);

  let mut entries = Vec::new();
  let mut list = tokio::fs::read_dir(base).await?;

  while let Ok(Some(entry)) = list.next_entry().await {
    let name = entry.file_name().to_string_lossy().to_string();
    let is_dir = entry.file_type().await?.is_dir();
    entries.push(Entry { name, is_dir });
  }

  entries.sort_by(|a, b| {
    if a.is_dir && b.is_dir {
      a.name.cmp(&b.name)
    } else if a.is_dir {
      Ordering::Less
    } else if b.is_dir {
      Ordering::Greater
    } else {
      a.name.cmp(&b.name)
    }
  });

  let globals = liquid::object!({
    "path": resolved.sanitized.to_string_lossy(),
    "entries":entries
  });

  let page = INDEX_TEMPLATE.render(&globals).map_err(|_| StaticError::Template)?;

  Ok(Response::builder().status(StatusCode::OK).body(page.into()).unwrap())
}

#[inline]
fn decode_percents(string: &str) -> String {
  percent_encoding::percent_decode_str(string)
    .decode_utf8_lossy()
    .into_owned()
}

fn normalize_path(path: &Path) -> PathBuf {
  path.components().fold(PathBuf::new(), |mut result, p| match p {
    Component::Normal(x) => {
      // Parse again to prevent a malicious component containing
      // a Windows drive letter, e.g.: `/anypath/c:/windows/win.ini`
      if Path::new(&x).components().all(|c| matches!(c, Component::Normal(_))) {
        result.push(x);
      }
      result
    }
    Component::ParentDir => {
      result.pop();
      result
    }
    _ => result,
  })
}

/// Resolved request path.
pub(crate) struct RequestedPath {
  /// Sanitized path of the request.
  pub(crate) sanitized: PathBuf,
}

impl RequestedPath {
  /// Resolve the requested path to a full filesystem path, limited to the root.
  pub(crate) fn resolve(request_path: &str) -> Self {
    let request_path = PathBuf::from(decode_percents(request_path));
    RequestedPath {
      sanitized: normalize_path(&request_path),
    }
  }
}
