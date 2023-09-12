#[cfg(test)]
mod test {
  use anyhow::{bail, Result};
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

  fn reset_dir() -> Result<()> {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(crate_dir)?;

    Ok(())
  }

  fn run_tests(path: &str) -> Result<()> {
    let bin = build_wick()?;
    let cwd = std::env::current_dir().unwrap();
    println!("cwd: {}", cwd.display());

    std::env::set_var("TRYCMD", "dump");

    set_dir()?;
    let files = glob::glob(path).unwrap().collect::<Vec<_>>();
    if files.is_empty() {
      bail!("No files found for pattern: {}", path);
    }
    for file in files {
      set_dir()?;
      let trycmd = trycmd::TestCases::new();

      trycmd
        .case(file.unwrap())
        .default_bin_name("wick")
        .default_bin_path(bin.path())
        .register_bin("wick", bin.path());

      trycmd.run();
      reset_dir()?;
    }

    reset_dir()?;
    Ok(())
  }

  #[test]
  fn wick_run() -> Result<()> {
    run_tests("integration-tests/cli-tests/tests/cmd/run/*.toml")?;
    Ok(())
  }

  mod integration_test {
    use super::*;

    #[test]
    fn db_tests() -> Result<()> {
      run_tests("integration-tests/cli-tests/tests/cmd/db/*.toml")?;
      Ok(())
    }

    #[test]
    fn wick_install() -> Result<()> {
      run_tests("integration-tests/cli-tests/tests/cmd/install/*.toml")?;
      Ok(())
    }
  }
}
