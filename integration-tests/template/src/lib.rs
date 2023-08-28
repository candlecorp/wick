#[cfg(test)]
mod slow_test {

  use cargo_generate::{GenerateArgs, TemplatePath};
  use toml_edit::Document;

  #[tokio::test]
  async fn test_generate() {
    let wsdir = std::env::current_dir().unwrap().join("../../");
    let test_dir = wsdir.join("templates/rust").canonicalize().unwrap();
    println!("wsdir: {}", test_dir.display());
    let tempdir = std::env::temp_dir().join("wick-test");
    println!("tempdir: {}", tempdir.display());
    if tempdir.exists() {
      println!("removing tempdir: {}", tempdir.display());
      std::fs::remove_dir_all(&tempdir).unwrap();
    }

    let template_path = TemplatePath {
      path: Some(test_dir.to_string_lossy().to_string()),
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

    let template_cargo_toml = tempdir.join(&name).join("Cargo.toml");

    let toml = std::fs::read_to_string(&template_cargo_toml).expect("could not read Cargo.toml");
    let mut doc = toml.parse::<Document>().expect("invalid toml");
    doc["dependencies"]["wick-component"]
      .as_inline_table_mut()
      .expect("wick-component not found")
      .remove("git");
    doc["dependencies"]["wick-component"]["path"] =
      toml_edit::value(wsdir.join("crates/wick/wick-component").to_string_lossy().to_string());
    doc["build-dependencies"]["wick-component-codegen"]
      .as_inline_table_mut()
      .expect("wick-component-codegen not found")
      .remove("git");
    doc["build-dependencies"]["wick-component-codegen"]["path"] = toml_edit::value(
      wsdir
        .join("crates/wick/wick-component-codegen")
        .to_string_lossy()
        .to_string(),
    );
    println!("modified toml: {}", doc);
    std::fs::write(&template_cargo_toml, doc.to_string()).expect("could not write Cargo.toml");

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

    let mut bin = tokio::process::Command::new("cargo");
    bin.arg("run");
    bin.arg("--manifest-path=../../Cargo.toml");
    bin.arg("-pwick-cli");
    bin.arg("--");
    bin.arg("invoke");
    bin.arg(tempdir.join(name).join("component.wick"));
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
