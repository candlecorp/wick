use std::process::Command;

fn main() {
  tonic_build::configure()
    .out_dir("src/generated")
    .file_descriptor_set_path("src/generated/descriptors.bin")
    .compile(&["proto/wick.proto"], &["proto"])
    .unwrap();

  let fmt = Command::new("cargo")
    .args(["+nightly", "fmt", "--", "src/generated/wick.rs"])
    .status()
    .expect("Failed to run cargo fmt on generated protobuf files.");

  if !fmt.success() {
    // This can happen on minimally setup machines and is not a problem on its own.
    println!("Could not format protobuf files");
  }
}
