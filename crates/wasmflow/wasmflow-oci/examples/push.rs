use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use oci_distribution::client::{Config, ImageLayer};
use oci_distribution::manifest::OciImageManifest;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;

#[derive(Parser, Default)]
struct Options {
  #[clap(action)]
  reference: String,

  #[clap(action)]
  path: PathBuf,

  #[clap(action)]
  media_type: String,

  #[clap(long, action)]
  manifest: Option<PathBuf>,

  #[clap(long, action)]
  config: Option<PathBuf>,

  #[clap(long, action)]
  insecure: Vec<String>,

  #[clap(long, env = "OCI_USERNAME", action)]
  username: Option<String>,

  #[clap(long, env = "OCI_PASSWORD", action)]
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

  let bytes = std::fs::read(opts.path)?;

  let manifest: Option<OciImageManifest> = match opts.manifest {
    Some(path) => Some(serde_json::from_slice(&std::fs::read(path)?)?),
    None => None,
  };

  let configdata = match opts.config {
    Some(path) => std::fs::read(path)?,
    None => b"{}".to_vec(),
  };

  let config = Config::new(
    configdata,
    oci_distribution::manifest::IMAGE_CONFIG_MEDIA_TYPE.to_owned(),
    None,
  );

  let imagedata = vec![ImageLayer {
    data: bytes,
    media_type: opts.media_type,
    annotations: None,
  }];

  let digest = c
    .push(
      &Reference::from_str(&opts.reference)?,
      &imagedata,
      config,
      &auth,
      manifest,
    )
    .await?;

  println!("Manifest url: {}", digest.manifest_url);
  println!("Config url: {:?}", digest.config_url);

  Ok(())
}
