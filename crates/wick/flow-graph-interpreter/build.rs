fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=component.yaml");
  // wick_component_codegen::configure().op_traits(false).generate("component.yaml")?;

  wick_component_codegen::configure()
    .out_dir("src/generated")
    .op_traits(false)
    .components(false)
    .generate("component.yaml")?;

  let fmt = std::process::Command::new("cargo")
    .args(["+nightly", "fmt", "--", "src/generated/mod.rs"])
    .status()
    .expect("Failed to run cargo fmt on generated protobuf files.");

  if !fmt.success() {
    // This can happen on minimally setup machines and is not a problem on its own.
    println!("Could not format protobuf files");
  }
  Ok(())
}
