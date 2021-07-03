#[actix_rt::test]
async fn run_log() -> vino_cli::Result<()> {
  let output = test_bin::get_test_bin("vino")
    .env_clear()
    .args(&[
      "run",
      "./manifests/log.vino",
      "{\"schem_input\": \"testing123\"}",
      "--trace",
    ])
    .output()
    .expect("Failed to start my_binary");

  println!("{}", String::from_utf8_lossy(&output.stderr));

  assert_eq!(
    String::from_utf8_lossy(&output.stdout),
    "Logger: testing123\n{\"schem_output\":\"testing123\"}\n"
  );

  Ok(())
}
