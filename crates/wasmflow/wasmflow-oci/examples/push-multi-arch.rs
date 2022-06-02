use std::path::PathBuf;

use clap::Parser;
use oci_distribution::secrets::RegistryAuth;
use wasmflow_oci::{parse_reference, ArchitectureMap, MultiArchManifest};

#[derive(Parser, Default)]
struct Options {
  reference: String,

  directory: PathBuf,

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
  let mut client = oci_distribution::Client::new(config);

  let auth = match (opts.username, opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username, password),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };

  let bytes = std::fs::read(opts.directory.join("manifest.yml"))?;
  let pushmanifest: MultiArchManifest = serde_yaml::from_slice(&bytes)?;
  let mut archmap = ArchitectureMap::default();
  for entry in pushmanifest.artifacts {
    let archive_bytes = Vec::new();
    let mut archive = tar::Builder::new(archive_bytes);
    archive.append_path_with_name(opts.directory.join(entry.path), "main.bin")?;
    archive.finish()?;
    let archive_bytes = archive.into_inner()?;
    archmap.add(entry.os, entry.arch, archive_bytes, None)
  }

  let reference = parse_reference(&opts.reference)?;

  wasmflow_oci::push_multi_arch(&mut client, &auth, &reference, archmap).await?;

  Ok(())
}
