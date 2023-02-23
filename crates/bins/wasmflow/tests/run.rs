#[tokio::test]
async fn wasmflow_run() {
  let output = test_bin::get_test_bin("wasmflow")
    .env_clear()
    .args([
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
    "Logger: testing123\n{\"metadata\":{\"index\":0,\"extra\":null},\"extra\":{\"done\":false,\"stream\":\"schem_output\"},\"payload\":{\"Ok\":[170,116,101,115,116,105,110,103,49,50,51]}}\n"
  );
}

#[test]
fn cli_tests() {
  // trycmd::TestCases::new().case("tests/cmd/**/*.toml");
}
