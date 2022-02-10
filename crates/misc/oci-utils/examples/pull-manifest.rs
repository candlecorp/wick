use std::str::FromStr;

use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
  reference: String,

  #[structopt(long)]
  insecure: Vec<String>,

  #[structopt(long, env = "OCI_USERNAME")]
  username: Option<String>,

  #[structopt(long, env = "OCI_PASSWORD")]
  password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env_logger::init();
  let opts = Options::from_args();
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
