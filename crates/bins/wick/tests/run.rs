#[tokio::test]
async fn wick_run() {
  let output = test_bin::get_test_bin("wick")
    .env_clear()
    .args([
      "invoke",
      "./manifests/log.wafl",
      "logger",
      "--data=schem_input=\"testing123\"",
      "--trace",
    ])
    .output()
    .expect("bin");

  println!("{}", String::from_utf8_lossy(&output.stderr));

  assert_eq!(
    String::from_utf8_lossy(&output.stdout)
      .split_terminator('\n')
      .next()
      .unwrap(),
    "Logger: testing123"
  );
}

#[test]
fn cli_tests() {
  trycmd::TestCases::new().case("tests/cmd/**/*.toml");
}
