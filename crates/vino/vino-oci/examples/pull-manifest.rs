use std::str::FromStr;

use clap::Parser;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;

#[derive(Parser)]
struct Options {
  reference: String,

  #[clap(long)]
  insecure: Vec<String>,

  #[clap(long, env = "OCI_USERNAME")]
  username: Option<String>,

  #[clap(long, env = "OCI_PASSWORD")]
  password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env_logger::init();
  let opts = Options::parse();
  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(opts.insecure);
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut c = oci_distribution::Client::new(config);

  let auth = match (opts.username, opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username, password),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };
  let (manifest, _config_digest, config) = c
    .pull_manifest_and_config(&Reference::from_str(&opts.reference)?, &auth)
    .await?;
  println!("{}", serde_json::to_string_pretty(&manifest)?);
  println!(
    "{}",
    serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&config)?)?
  );
  // println!("{:?}", manifest);
  Ok(())
}

#[cfg(test)]
mod test {
  #[test]
  fn verify_options() {
    use clap::IntoApp;
    super::Options::command().debug_assert();
  }
}
