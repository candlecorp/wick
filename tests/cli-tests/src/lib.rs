#[cfg(test)]
mod test {
  use anyhow::Result;
  use escargot::CargoRun;

  fn build_wick() -> Result<CargoRun> {
    println!("building wick or using cached binary");
    let bin = escargot::CargoBuild::new()
      .manifest_path("../../Cargo.toml")
      .package("wick-cli")
      .bin("wick")
      .run()?;
    println!("using binary at: {:?}", bin.path().display());
    Ok(bin)
  }

  fn set_dir() -> Result<()> {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(crate_dir.join("../.."))?;
    println!("current working directory is:  {:?}", std::env::current_dir()?);
    Ok(())
  }

  fn run_tests(path: &str) -> Result<()> {
    let bin = build_wick()?;
    set_dir()?;

    let trycmd = trycmd::TestCases::new();
    trycmd
      .case(path)
      .default_bin_name("wick")
      .default_bin_path(bin.path())
      .register_bin("wick", bin.path());

    trycmd.run();
    Ok(())
  }

  mod integration_test {
    use super::*;

    #[test]
    fn db_tests() -> Result<()> {
      run_tests("tests/cli-tests/tests/cmd/db/*.toml")?;
      Ok(())
    }
  }
}
