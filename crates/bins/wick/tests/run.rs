static DIR: &str = "run";

#[rstest::rstest]
#[case("anonymous-component.toml")]
#[case("imported-component.toml")]
#[case("stdin.toml")]
#[case("file-reader.toml")]
#[case("file-reader-lockdown-fail.toml")]
#[case("file-reader-lockdown-pass.toml")]
#[case("file-reader-lockdown-pass-wildcard-dir.toml")]
#[case("file-reader-lockdown-pass-wildcard-components.toml")]
fn wick_run(#[case] file: &'static str) {
  let kind = "unit";
  let file = format!("tests/{}/{}/{}", DIR, kind, file);

  trycmd::TestCases::new().case(file);
}

mod integration_test {
  use super::DIR;
  #[rstest::rstest]
  #[case("postgres.toml")]
  fn wick_run(#[case] file: &'static str) {
    let kind = "integration";
    let file = format!("tests/{}/{}/{}", DIR, kind, file);
    trycmd::TestCases::new().case(file);
  }
}
