use std::path::PathBuf;
use std::str::FromStr;

use oci_distribution::Reference;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs;

use crate::Error;

/// Parse a `&str` as a Reference.
pub fn parse_reference(reference: &str) -> Result<Reference, Error> {
  oci_distribution::Reference::from_str(reference)
    .map_err(|e| Error::OCIParseError(reference.to_owned(), e.to_string()))
}

/// Parse a `&str` as a Reference and return the protocol to use.
pub fn parse_reference_and_protocol(
  reference: &str,
  allowed_insecure: &[String],
) -> Result<(Reference, oci_distribution::client::ClientProtocol), Error> {
  let reference =
    Reference::from_str(reference).map_err(|e| Error::OCIParseError(reference.to_owned(), e.to_string()))?;
  let insecure = allowed_insecure.contains(&reference.registry().to_owned());
  Ok((
    reference,
    if insecure {
      oci_distribution::client::ClientProtocol::Http
    } else {
      oci_distribution::client::ClientProtocol::Https
    },
  ))
}

pub(crate) fn get_cache_directory(input: &str, basedir: Option<PathBuf>) -> Result<PathBuf, Error> {
  let image_ref_result = Reference::from_str(input);
  let image_ref = match image_ref_result {
    Ok(image_ref) => image_ref,
    Err(_) => {
      return Err(Error::InvalidReference(input.to_owned()));
    }
  };

  let registry = image_ref.registry().split(':').collect::<Vec<&str>>()[0];
  let org = image_ref.repository().split('/').collect::<Vec<&str>>()[0];
  let repo = image_ref.repository().split('/').collect::<Vec<&str>>()[1];
  let version = image_ref.tag().ok_or(Error::NoName)?;

  let parts = vec![registry, org, repo, version];

  if parts.len() != 4 {
    return Err(Error::InvalidReference(input.to_owned()));
  }
  let mut basedir = basedir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
  debug!(path = %basedir.display(), "Base cache directory");

  // Create the wick_components directory if it doesn't exist
  basedir.push(wick_xdg::Cache::Assets.basedir());
  let target_dir = basedir.join(registry).join(org).join(repo).join(version);
  Ok(target_dir)
}

pub(crate) async fn create_directory_structure(dir: PathBuf) -> Result<PathBuf, Error> {
  fs::create_dir_all(&dir)
    .await
    .map_err(|e| Error::CreateDir(dir.clone(), e))?;

  debug!(path = %dir.display(), "Directory created");

  Ok(dir)
}

static WICK_REF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+/\w+:(\d+\.\d+\.\d+(-\w+)?|latest)?$").unwrap());

pub fn is_wick_package_reference(loc: impl AsRef<str>) -> bool {
  WICK_REF_REGEX.is_match(loc.as_ref())
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use anyhow::Result;

  use super::*;

  #[test]
  fn test_directory_structure() {
    let input = "localhost:5555/test/integration:0.0.3";

    let expected_dir = Path::new("/remote/localhost/test/integration/0.0.3");
    let result = get_cache_directory(input, Some("/".into())).unwrap();
    assert_eq!(result, expected_dir);

    let input = "example.com/myorg/myrepo:1.0.0";
    let expected_dir = Path::new("/remote/example.com/myorg/myrepo/1.0.0");
    let result = get_cache_directory(input, Some("/".into())).unwrap();
    assert_eq!(result, expected_dir);
  }

  #[test]
  fn test_good_wickref() -> Result<()> {
    assert!(is_wick_package_reference("this/that:1.2.3"));
    assert!(is_wick_package_reference("this/that:1.2.3"));
    assert!(is_wick_package_reference("1alpha/2alpha:0000.2222.9999"));
    assert!(is_wick_package_reference("a_b_c_1/1_2_3_a:1.2.999-alpha"));
    assert!(is_wick_package_reference("this/that:latest"));
    assert!(is_wick_package_reference(
      "registry.candle.dev/fawadasaurus/serve_http_component:0.0.1"
    ));

    Ok(())
  }

  #[test]
  fn test_bad_wickref() -> Result<()> {
    assert!(!is_wick_package_reference("not/this:bad_tag"));

    Ok(())
  }
}
