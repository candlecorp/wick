mod integration_test {

  use std::path::Path;
  use std::time::{SystemTime, UNIX_EPOCH};

  use anyhow::Result;
  use walkdir::WalkDir;
  use wick_oci_utils::package::PackageFile;
  use wick_oci_utils::OciOptions;
  use wick_package::WickPackage;

  #[test_logger::test(tokio::test)]
  async fn test_push_and_pull_wick_package() -> Result<()> {
    let crate_dir = format!("{}/./tests/files", env!("CARGO_MANIFEST_DIR"));

    let host = std::env::var("DOCKER_REGISTRY").unwrap();
    let tempdir = std::env::temp_dir().join("test_push_and_pull_wick_package");
    let _ = tokio::fs::remove_dir_all(&tempdir).await;
    tokio::fs::create_dir_all(&tempdir).await.unwrap();

    println!("Using tempdir: {:?}", tempdir);
    let options = OciOptions::default()
      .overwrite(true)
      .base_dir(Some(tempdir.clone()))
      .allow_insecure(vec![host.to_owned()]);

    // Run the push operation
    let package_path = Path::new("./tests/files/jinja.wick");
    println!("Package path: {:?}", package_path);
    let mut package = WickPackage::from_path(package_path).await.unwrap();
    let num_files = package.list_files().len();
    assert_eq!(num_files, 4, "Mismatch in hard coded number of files");

    // necessary to clone our WickPackage because push() consumes our contents and we
    // want to test the original bytes post-push.
    let expected = package.clone();
    let test_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let reference = expected
      .tagged_reference(&test_timestamp.as_secs().to_string())
      .unwrap();
    let push_result = package.push(&reference, &options).await;
    if push_result.is_err() {
      panic!("Failed to push WickPackage: {:?}", push_result);
    };
    drop(package); // dropping it here to make sure tests use the clone `expected` instead.

    // Run the pull operation
    let pulled_package = WickPackage::pull(&reference, &options).await?;

    // Check if the pulled package is the same as the pushed one
    assert_eq!(
      expected.list_files().len(),
      pulled_package.list_files().len(),
      "Mismatch in number of files between packages"
    );
    let mut root_dir = pulled_package.path().to_owned();
    root_dir.pop();
    println!("Counting files in pulled directory: {}", root_dir.display());
    let num_files = WalkDir::new(root_dir).into_iter().count();

    assert_eq!(num_files, 11, "Mismatch in number of files in the pulled package");

    let pushed_files: Vec<&PackageFile> = expected.list_files();
    let pulled_files: Vec<&PackageFile> = pulled_package.list_files();

    // Sort both the pushed_files and pulled_files by path
    let mut pushed_files_sorted = pushed_files.clone();
    let mut pulled_files_sorted = pulled_files.clone();
    pushed_files_sorted.sort_by_key(|file| file.path());
    pulled_files_sorted.sort_by_key(|file| file.path());

    for (pushed_file, pulled_file) in pushed_files_sorted.iter().zip(pulled_files_sorted.iter()) {
      let pushed_file_path = pushed_file.path().to_str().unwrap().trim_start_matches(&crate_dir);
      let pulled_file_path = pulled_file
        .path()
        .to_str()
        .unwrap()
        .trim_start_matches(tempdir.to_str().unwrap());
      assert_eq!(pushed_file_path, pulled_file_path, "Mismatch in file paths");
      //if pushed_file.path() ends with .tar.gz, don't compare hashes
      if pushed_file.path().to_str().unwrap().ends_with(".tar.gz") {
        continue;
      }
      println!("Comparing hashes for file: {:?}", pushed_file.path());
      assert_eq!(pushed_file.hash(), pulled_file.hash(), "Mismatch in file hashes");
      assert_eq!(
        pushed_file.media_type(),
        pulled_file.media_type(),
        "Mismatch in file media types"
      );
      assert_eq!(
        pushed_file.contents(),
        pulled_file.contents(),
        "Mismatch in file contents"
      );
    }
    Ok(())
  }
}
