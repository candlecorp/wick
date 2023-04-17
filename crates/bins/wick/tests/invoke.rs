static DIR: &str = "invoke";

#[test]
fn wick_invoke() {
  let kind = "unit";
  trycmd::TestCases::new()
    .case(format!("tests/{}/{}/*.toml", kind, DIR))
    .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
}

mod integration_tests {
  use super::DIR;
  #[test]
  fn wick_invoke() {
    let kind = "integration";
    trycmd::TestCases::new()
      .case(format!("tests/{}/{}/*.toml", kind, DIR))
      .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
  }
}
