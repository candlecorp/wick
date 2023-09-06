use std::path::{Path, PathBuf};

use oci_distribution::Reference;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs;

use crate::Error;

// Originally borrowed from oci-distribution who doesn't export it...
// pub static REFERENCE_REGEXP: &str = r"^((?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:(?:\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))+)?(?::[0-9]+)?/)?[a-z0-9]+(?:(?:(?:[._]|__|[-]*)[a-z0-9]+)+)?(?:(?:/[a-z0-9]+(?:(?:(?:[._]|__|[-]*)[a-z0-9]+)+)?)+)?)(?::([\w][\w.-]{0,127}))?(?:@([A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][[:xdigit:]]{32,}))?$";

// ... with some minor changes that reduce the cost of the regex by >80% at the risk of some outlier false positives:
pub static REFERENCE_REGEXP: &str = r"^((?:(?:[[:alnum:]]+)(?:(?:\.(?:[[:alnum:]]+))+)?(?::[[:digit:]]+)?/)?[[:lower:][:digit:]]+(?:(?:(?:[._]|__|[-]*)[[:lower:][:digit:]]+)+)?(?:(?:/[[:lower:][:digit:]]+(?:(?:(?:[._]|__|[-]*)[[:lower:][:digit:]]+)+)?)+)?)(?::([\w][\w.-]*))?(?:@([[:alpha:]][[:alnum:]]*(?:[-_+.][[:alpha:]][[:alnum:]]*)*[:][[:xdigit:]]+))?$";

static RE: Lazy<Regex> = Lazy::new(|| {
  regex::RegexBuilder::new(REFERENCE_REGEXP)
    .size_limit(10 * (1 << 21))
    .build()
    .unwrap()
});

pub const DEFAULT_REGISTRY: &str = "registry.candle.dev";

/// Check if a &str is an OCI reference.
pub fn is_oci_reference(reference: &str) -> bool {
  RE.is_match(reference)
}

/// Parse a `&str` as an OCI Reference.
pub fn parse_reference(reference: &str) -> Result<Reference, Error> {
  let captures = RE
    .captures(reference)
    .ok_or(Error::InvalidReferenceFormat(reference.to_owned()))?;
  let name = &captures[1];
  let tag = captures.get(2).map(|m| m.as_str().to_owned());
  let digest = captures.get(3).map(|m| m.as_str().to_owned());

  let (registry, repository) = split_domain(name);

  if let Some(tag) = tag {
    Ok(oci_distribution::Reference::with_tag(registry, repository, tag))
  } else if let Some(digest) = digest {
    Ok(oci_distribution::Reference::with_digest(registry, repository, digest))
  } else {
    Err(Error::NoTagOrDigest(reference.to_owned()))
  }
}

// Also borrowed from oci-distribution who borrowed it from the go docker implementation.
fn split_domain(name: &str) -> (String, String) {
  match name.split_once('/') {
    None => (DEFAULT_REGISTRY.to_owned(), name.to_owned()),
    Some((left, right)) => {
      if !(left.contains('.') || left.contains(':')) && left != "localhost" {
        (DEFAULT_REGISTRY.to_owned(), name.to_owned())
      } else {
        (left.to_owned(), right.to_owned())
      }
    }
  }
}

/// Parse a `&str` as a Reference and return the protocol to use.
pub fn parse_reference_and_protocol(
  reference: &str,
  allowed_insecure: &[String],
) -> Result<(Reference, oci_distribution::client::ClientProtocol), Error> {
  let reference = parse_reference(reference)?;

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

pub fn get_cache_directory<T: AsRef<Path>>(input: &str, basedir: T) -> Result<PathBuf, Error> {
  let image_ref = parse_reference(input)?;

  let registry = image_ref
    .registry()
    .split_once(':')
    .map_or(image_ref.registry(), |(reg, _port)| reg);
  let (org, repo) = image_ref.repository().split_once('/').ok_or(Error::OCIParseError(
    input.to_owned(),
    "repository was not in org/repo format".to_owned(),
  ))?;

  let version = image_ref.tag().ok_or(Error::NoName)?;

  // Create the wick_components directory if it doesn't exist
  let target_dir = basedir.as_ref().join(registry).join(org).join(repo).join(version);
  Ok(target_dir)
}

pub(crate) async fn create_directory_structure(dir: &Path) -> Result<(), Error> {
  fs::create_dir_all(&dir)
    .await
    .map_err(|e| Error::CreateDir(dir.to_path_buf(), e))?;

  debug!(path = %dir.display(), "Directory created");

  Ok(())
}

static WICK_REF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+/\w+:(\d+\.\d+\.\d+(-\w+)?|latest)?$").unwrap());

pub fn is_wick_package_reference(loc: &str) -> bool {
  WICK_REF_REGEX.is_match(loc)
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use anyhow::Result;

  use super::*;

  #[rstest::rstest]
  #[case("localhost:5555/test/integration:0.0.3", "", "localhost/test/integration/0.0.3")]
  #[case(
    "example.com/myorg/myrepo:1.0.0",
    "/foo/bar",
    "/foo/bar/example.com/myorg/myrepo/1.0.0"
  )]
  #[case("org/myrepo:1.0.1", "", "registry.candle.dev/org/myrepo/1.0.1")]
  fn directory_structure_positive(#[case] input: &str, #[case] basedir: &str, #[case] expected: &str) {
    let expected_dir = Path::new(expected);
    let result = get_cache_directory(input, basedir).unwrap();
    assert_eq!(result, expected_dir);
  }
  #[rstest::rstest]
  #[case("example.com/myrepo:1.0.0")]
  #[case("example.com/org/myrepo")]
  #[case("example.com/myrepo")]
  #[case("example.com:5000/myrepo:1.0.0")]
  #[case("example.com:5000/org/myrepo")]
  #[case("example.com:5000/myrepo")]
  #[case("myrepo:1.0.0")]
  #[case("org/myrepo")]
  #[case("myrepo")]
  fn directory_structure_negative(#[case] input: &str) {
    let result = get_cache_directory(input, "");
    println!("{:?}", result);
    assert!(result.is_err());
  }

  #[test]
  fn test_good_wickref() -> Result<()> {
    assert!(is_wick_package_reference("this/that:1.2.3"));
    assert!(is_wick_package_reference("this/that:1.2.3"));
    assert!(is_wick_package_reference("1alpha/2alpha:0000.2222.9999"));
    assert!(is_wick_package_reference("a_b_c_1/1_2_3_a:1.2.999-alpha"));
    assert!(is_wick_package_reference("this/that:latest"));

    Ok(())
  }

  #[test]
  fn test_bad_wickref() -> Result<()> {
    assert!(!is_wick_package_reference("not/this:bad_tag"));

    Ok(())
  }
}
