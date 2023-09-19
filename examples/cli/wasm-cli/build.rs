fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=component.wick");
    #[cfg(not(feature = "localgen"))]
    wick_component_codegen::configure()
        .raw(true)
        .generate("component.wick")?;

    #[cfg(feature = "localgen")]
    {
        wick_component_codegen::configure()
            .out_dir("src/generated")
            .raw(true)
            .generate("component.wick")?;

        let fmt = std::process::Command::new("cargo")
            .args(["+nightly", "fmt", "--", "src/generated/mod.rs"])
            .status()
            .expect("Failed to run cargo fmt on generated files.");

        if !fmt.success() {
            // This can happen on minimally setup machines and is not a problem on its own.
            println!("Could not format generated files");
        }
    }
    Ok(())
}