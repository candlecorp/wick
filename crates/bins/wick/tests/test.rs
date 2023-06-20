static DIR: &str = "test";

#[rstest::rstest]
#[case("wasm.toml")]
fn wick_run(#[case] file: &'static str) {
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
