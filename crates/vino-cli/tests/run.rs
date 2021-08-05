#[actix::test]
async fn vino_run() {
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
    "Logger: testing123\n{\"value\":\"testing123\"}\n"
  );
}
