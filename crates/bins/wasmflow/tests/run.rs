#[tokio::test]
async fn vino_run() {
  let output = test_bin::get_test_bin("wasmflow")
    .env_clear()
    .args(&[
      "invoke",
      "./manifests/log.vino",
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
