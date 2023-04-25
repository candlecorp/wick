#[cfg(test)]
mod slow_test {

  use cargo_generate::{GenerateArgs, TemplatePath};

  #[tokio::test]
  async fn test_generate() {
    let wsdir = std::env::current_dir()
      .unwrap()
      .join("../../templates/rust")
      .canonicalize()
      .unwrap();
    println!("wsdir: {}", wsdir.display());
    let tempdir = std::env::temp_dir().join("wick-test");
    println!("tempdir: {}", tempdir.display());
    if tempdir.exists() {
      println!("removing tempdir: {}", tempdir.display());
      std::fs::remove_dir_all(&tempdir).unwrap();
    }

    let template_path = TemplatePath {
      path: Some(wsdir.to_string_lossy().to_string()),
      ..Default::default()
    };
    let name = "wick-test".to_owned();

    let args = GenerateArgs {
      name: Some(name.clone()),
      destination: Some(tempdir.clone()),
      template_path,
      ..Default::default()
    };

    let result = cargo_generate::generate(args).unwrap();
    println!("result: {:?}", result);
    let cmd = tokio::process::Command::new("just")
      .current_dir(tempdir.join(&name))
      .args(["build"])
      .output()
      .await
      .unwrap();
    let output = String::from_utf8_lossy(&cmd.stdout);
    if !cmd.status.success() {
      println!("output: {:?}", output);
      panic!("stderr: {}", String::from_utf8_lossy(&cmd.stderr));
    }

    let mut bin = tokio::process::Command::from(test_bin::get_test_bin("cargo"));
    bin.arg("run");
    bin.arg("-pwick-cli");
    bin.arg("--");
    bin.arg("invoke");
    bin.arg(tempdir.join(name).join("component.yaml"));
    bin.arg("add");
    bin.arg("--");
    bin.arg("--left=10");
    bin.arg("--right=2219");
    println!("bin: {:?}", bin);
    let output = bin.output().await.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
      println!("output: {:?}", stdout);
      panic!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert_eq!(stdout, "{\"payload\":{\"value\":2229},\"port\":\"output\"}\n");
  }
}
