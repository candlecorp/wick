#[test]
fn run_tests() {
  trycmd::TestCases::new().case("tests/cmd/run/*.toml");
}
