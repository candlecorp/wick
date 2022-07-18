use std::process::Command;

fn main() {
  println!("cargo:rerun-if-changed=proto/wasmflow.proto");
  println!("cargo:rustc-env=OUT_DIR=generated");
  tonic_build::configure()
    .out_dir("src/generated")
    .file_descriptor_set_path("src/generated/descriptors.bin")
    .compile(&["proto/wasmflow.proto"], &["proto"])
    .unwrap();

  let fmt = Command::new("cargo")
    .args(&["+nightly", "fmt", "--", "src/generated/wasmflow.rs"])
    .status()
    .expect("Failed to run cargo fmt on generated protobuf files.");
  assert!(fmt.success(), "Can't format protobuf files");
}
