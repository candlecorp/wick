static DIR: &str = "new";

#[rstest::rstest]
#[case("app.toml")]
#[case("app-cli.toml")]
#[case("app-http.toml")]
#[case("app-time.toml")]
#[case("component-composite.toml")]
#[case("component-sql.toml")]
#[case("component-http.toml")]
#[case("component-wasm.toml")]
fn wick_new(#[case] file: &'static str) {
  let kind = "unit";
  let file = format!("tests/{}/{}/{}", DIR, kind, file);

  trycmd::TestCases::new().case(file);
}

// mod integration_test {
//   use super::DIR;
//   #[rstest::rstest]
//   fn wick_run(#[case] file: &'static str) {
//     let kind = "integration";
//     let file = format!("tests/{}/{}/{}", kind, DIR, file);
//     trycmd::TestCases::new().case(file);
//   }
// }
