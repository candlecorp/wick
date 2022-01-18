fn main() {
  println!("cargo:rerun-if-changed=proto/vino.proto");
  println!("cargo:rustc-env=OUT_DIR=generated");
  tonic_build::configure()
    .out_dir("src/generated")
    .file_descriptor_set_path("src/generated/descriptors.bin")
    .compile(&["proto/vino.proto"], &["proto"])
    .unwrap();
}
