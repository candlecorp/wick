//! A module for getting the crate binary in an integration test.
//!
//! If you are writing a command-line interface app then it is useful to write
//! an integration test that uses the binary. You most likely want to launch the
//! binary and inspect the output. This module lets you get the binary so it can
//! be tested.
//!
//! # Examples
//!
//! basic usage:
//!
//! ```no_run
//! # async fn run() -> Result<(), std::io::Error> {
//!   let output = tokio_test_bin::get_test_bin("my_cli_app")
//!     .output().await?;
//!
//!   assert_eq!(
//!     String::from_utf8_lossy(&output.stdout),
//!     "Output from my CLI app!\n"
//!   );
//! # Ok(())
//! # }
//! ```
//!
//! Refer to the [`std::process::Command` documentation](https://doc.rust-lang.org/std/process/struct.Command.html)
//! for how to pass arguments, check exit status and more.

pub mod error;
pub use error::Error;

/// Returns the crate's binary as a `Command` that can be used for integration
/// tests.
///
/// # Arguments
///
/// * `bin_name` - The name of the binary you want to test.
///
/// # Remarks
///
/// It panics on error. This is by design so the test that uses it fails.
pub fn get_test_bin(bin_name: &str) -> tokio::process::Command {
  // Create full path to binary
  let mut path = get_test_bin_dir();
  path.push(bin_name);
  path.set_extension(std::env::consts::EXE_EXTENSION);

  assert!(path.exists());

  // Create command
  tokio::process::Command::new(path.into_os_string())
}

/// Returns the directory of the crate's binary.
///
/// # Remarks
///
/// It panics on error. This is by design so the test that uses it fails.
fn get_test_bin_dir() -> std::path::PathBuf {
  // Cargo puts the integration test binary in target/debug/deps
  let current_exe =
    std::env::current_exe().expect("Failed to get the path of the integration test binary");

  let current_dir = current_exe
    .parent()
    .expect("Failed to get the directory of the integration test binary");

  assert!(
    current_dir.ends_with("target/debug/deps") || current_dir.ends_with("target/release/deps")
  );

  let test_bin_dir = current_dir
    .parent()
    .expect("Failed to get the binary folder");
  test_bin_dir.to_owned()
}
