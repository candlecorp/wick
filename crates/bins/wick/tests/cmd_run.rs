static DIR: &str = "run";

#[test]
fn invoke_tests() {
  let kind = "unit";
  trycmd::TestCases::new()
    .case(format!("tests/{}/{}/*.toml", kind, DIR))
    .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
}

mod integration_tests {
  use super::DIR;
  #[test]
  fn invoke_tests() {
    let kind = "integration";
    trycmd::TestCases::new()
      .case(format!("tests/{}/{}/*.toml", kind, DIR))
      .case(format!("tests/{}/{}/*.trycmd", kind, DIR));
  }
}
