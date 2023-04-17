static DIR: &str = "run";

#[test]
fn wick_run() {
  let kind = "unit";
  trycmd::TestCases::new()
    .case(format!("tests/{}/{}/*.toml", kind, DIR))
    .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
}

mod integration_tests {
  use super::DIR;
  #[test]
  fn wick_run() {
    let kind = "integration";
    trycmd::TestCases::new()
      .case(format!("tests/{}/{}/*.toml", kind, DIR))
      .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
  }
}
