use std::path::{Path, PathBuf};
use std::str::FromStr;

use oci_distribution::Reference;
use tokio::fs;

use crate::Error;

#[allow(unused)]
pub(crate) async fn create_directory_structure(input: &str) -> Result<PathBuf, Error> {
  // Parse the input reference
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

  // put these 4 variables in a vector called parts
  let parts = vec![registry, org, repo, version];

  if parts.len() != 4 {
    return Err(Error::InvalidReference(input.to_owned()));
  }

  // Create the wick_components directory if it doesn't exist
  let base_dir = Path::new("./wick_components");
  fs::create_dir_all(&base_dir)
    .await
    .map_err(|e| Error::CreateDir(base_dir.to_path_buf(), e))?;

  // Create the required subdirectories
  let target_dir = base_dir.join(registry).join(org).join(repo).join(version);
  fs::create_dir_all(&target_dir)
    .await
    .map_err(|e| Error::CreateDir(target_dir.clone(), e))?;

  println!("Directory created: {}", target_dir.display());

  Ok(target_dir)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_create_directory_structure() {
    let input = "localhost:8888/test/integration:0.0.3";
    let expected_dir = Path::new("./wick_components/localhost/test/integration/0.0.3");
    let result = create_directory_structure(input).await.unwrap();
    assert_eq!(result, expected_dir);

    let input = "example.com/myorg/myrepo:1.0.0";
    let expected_dir = Path::new("./wick_components/example.com/myorg/myrepo/1.0.0");
    let result = create_directory_structure(input).await.unwrap();
    assert_eq!(result, expected_dir);
  }
}
