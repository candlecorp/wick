#[tokio::test]
async fn wasmflow_run() {
  let output = test_bin::get_test_bin("wasmflow")
    .env_clear()
    .args(&[
      "invoke",
      "./manifests/log.wafl",
      "--data=schem_input=\"testing123\"",
      "--trace",
    ])
    .output()
    .expect("bin");

  println!("{}", String::from_utf8_lossy(&output.stderr));

  assert_eq!(
    String::from_utf8_lossy(&output.stdout),
    "Logger: testing123\n{\"schem_output\":{\"value\":\"testing123\"}}\n"
  );
}

#[test]
fn cli_tests() {
  trycmd::TestCases::new().case("tests/cmd/**/*.toml");
}
