fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=component.yaml");
  wick_component_codegen::configure().generate("component.yaml")?;
  Ok(())
}
