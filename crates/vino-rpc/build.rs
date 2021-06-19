fn main() {
  println!("cargo:rerun-if-changed=proto/vino.proto");
  println!("cargo:rustc-env=OUT_DIR=generated");
  tonic_build::configure()
    .out_dir("src/generated")
    .compile(&["proto/vino.proto"], &["proto"])
    .unwrap();
}
