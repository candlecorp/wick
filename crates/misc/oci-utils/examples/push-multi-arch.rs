use std::path::PathBuf;

use oci_distribution::secrets::RegistryAuth;
use oci_utils::ArchitectureMap;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt, Default)]
struct Options {
  directory: PathBuf,

  tag: Option<String>,

  #[structopt(long)]
  insecure: Vec<String>,

  #[structopt(long, env = "OCI_USERNAME")]
  username: Option<String>,

  #[structopt(long, env = "OCI_PASSWORD")]
  password: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MultiArchManifest {
  registry: String,
  repo: String,
  artifacts: Vec<ArchManifestEntry>,
}

#[derive(Serialize, Deserialize)]
struct ArchManifestEntry {
  path: PathBuf,
  os: String,
  arch: String,
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

  oci_utils::push_multi_arch(
    &mut client,
    &auth,
    pushmanifest.registry,
    pushmanifest.repo,
    opts.tag,
    archmap,
  )
  .await?;

  Ok(())
}
