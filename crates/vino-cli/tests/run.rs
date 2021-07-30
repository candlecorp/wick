#[actix_rt::test]
async fn run_log() {
  let output = test_bin::get_test_bin("vino")
    .env_clear()
    .args(&[
      "run",
      "./manifests/log.vino",
      "{\"schem_input\": \"testing123\"}",
      "--trace",
    ])
    .output()
    .expect("bin");

  println!("{}", String::from_utf8_lossy(&output.stderr));

  assert_eq!(
    String::from_utf8_lossy(&output.stdout),
    "Logger: testing123\n{\"error_kind\":\"None\",\"value\":\"testing123\"}\n"
  );
}
