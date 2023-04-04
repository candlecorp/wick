#[test]
fn invoke_tests() {
  trycmd::TestCases::new().case("tests/cmd/invoke/unit-*.toml");
}

mod integration_tests {
  #[test]
  fn invoke_tests() {
    trycmd::TestCases::new().case("tests/cmd/invoke/integration-*.toml");
  }
}
