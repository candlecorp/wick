#[test]
fn invoke_tests() {
  trycmd::TestCases::new().case("tests/cmd/invoke/*.toml");
}
