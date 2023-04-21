fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = std::env::args().nth(1).unwrap();
  let dir = std::env::args().nth(2).unwrap();
  gen(&spec, &dir)?;
  Ok(())
}

fn gen(spec: &str, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
  println!("Generating code from {} in {}", spec, dir);
  wick_component_codegen::configure().out_dir(dir).generate(spec)?;

  let fmt = std::process::Command::new("cargo")
    .args(["+nightly", "fmt", "--"])
    .arg(format!("{}/mod.rs", dir))
    .status()
    .expect("Failed to run cargo fmt on generated files.");

  if !fmt.success() {
    // This can happen on minimally setup machines and is not a problem on its own.
    println!("Could not format generated files");
  }
  Ok(())
}
