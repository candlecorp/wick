mod integration_test {

  use std::path::Path;

  use wick_oci_utils::package::PackageFile;
  use wick_oci_utils::OciOptions;
  use wick_package::WickPackage;

  #[tokio::test]
  async fn test_push_and_pull_wick_package() {
    let host = std::env::var("DOCKER_HOST").unwrap();
    let tempdir = std::env::temp_dir();
    let options = OciOptions::default()
      .base_dir(Some(tempdir))
      .allow_insecure(vec!["localhost:8888".to_owned()]);

    // Run the push operation
    let package_path = Path::new("./tests/files/jinja.wick");
    let mut package = WickPackage::from_path(package_path).await.unwrap();
    // necessary to clone our WickPackage because push() consumes our contents and we
    // want to test the original bytes post-push.
    let expected = package.clone();
    let reference = format!("{}/test/integration:0.0.3", host);
    let push_result = package.push(&reference, &options).await;
    assert!(push_result.is_ok(), "Failed to push WickPackage");
    drop(package); // dropping it here to make sure tests use the clone `expected` instead.

    // Run the pull operation
    let pulled_package_result = WickPackage::pull(&reference, &options).await;
    println!("{:?}", pulled_package_result);
    assert!(pulled_package_result.is_ok(), "Failed to pull WickPackage");
    let pulled_package = pulled_package_result.unwrap();

    // Check if the pulled package is the same as the pushed one
    assert_eq!(
      expected.list_files().len(),
      pulled_package.list_files().len(),
      "Mismatch in number of files"
    );

    let pushed_files: Vec<&PackageFile> = expected.list_files();
    let pulled_files: Vec<&PackageFile> = pulled_package.list_files();
    for (pushed_file, pulled_file) in pushed_files.iter().zip(pulled_files.iter()) {
      assert_eq!(pushed_file.path(), pulled_file.path(), "Mismatch in file paths");
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
  }
}
