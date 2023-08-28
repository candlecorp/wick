fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=component.wick");
  wick_component_codegen::configure()
    .raw(true)
    .generate("component.wick")?;
  Ok(())
}
