use std::str::FromStr;

use clap::Parser;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};

#[derive(Parser)]
struct Options {
  reference: String,

  #[clap(long)]
  insecure: Vec<String>,

  #[clap(long)]
  os: Option<String>,

  #[clap(long)]
  arch: Option<String>,

  #[clap(long, env = "OCI_USERNAME")]
  username: Option<String>,

  #[clap(long, env = "OCI_PASSWORD")]
  password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env_logger::init();
  let opts = Options::parse();
  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(opts.insecure.clone());
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut client = Client::new(config);

  let auth = match (&opts.username, &opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username.clone(), password.clone()),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };

  let reference = Reference::from_str(&opts.reference)?;

  pull(&reference, &opts, &mut client, &auth).await?;

  Ok(())
}

#[async_recursion::async_recursion]
async fn pull(reference: &Reference, opts: &Options, client: &mut Client, auth: &RegistryAuth) -> anyhow::Result<()> {
  let (manifest, _) = client.pull_manifest(reference, auth).await?;
  let imagedata = client
    .pull(
      reference,
      auth,
      vec![
        oci_distribution::manifest::WASM_LAYER_MEDIA_TYPE,
        oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE,
        oci_distribution::manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE,
        oci_distribution::manifest::IMAGE_DOCKER_LAYER_TAR_MEDIA_TYPE,
        oci_distribution::manifest::IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE,
      ],
    )
    .await?;

  for (i, layer) in imagedata.layers.into_iter().enumerate() {
    match &manifest {
      oci_distribution::manifest::OciManifest::Image(manifest) => {
        if let Some(Some(annotations)) = manifest.layers.get(i).map(|l| &l.annotations) {
          if let Some(name) = annotations.get("org.opencontainers.image.title") {
            std::fs::write(name, layer.data)?;
          }
        } else {
          std::fs::write(format!("layer-{}.out", i), layer.data)?;
        }
      }
      oci_distribution::manifest::OciManifest::ImageIndex(manifest) => {
        if let (Some(os), Some(arch)) = (&opts.os, &opts.arch) {
          let mut valid_platforms = vec![];
          let platform_manifest = manifest.manifests.iter().find(|manifest| match &manifest.platform {
            Some(platform) => {
              valid_platforms.push(format!("{}-{}", platform.os, platform.architecture));
              &platform.os == os && &platform.architecture == arch
            }
            None => false,
          });
          if platform_manifest.is_none() {
            println!("Platform {}-{} not found", os, arch);
            println!("Valid platforms are: {:?}", valid_platforms);
          }
          let platform_manifest = platform_manifest.unwrap();
          let reference = Reference::with_digest(
            reference.registry().to_owned(),
            reference.repository().to_owned(),
            platform_manifest.digest.clone(),
          );
          println!("Platform reference: {}", reference);

          pull(&reference, opts, client, auth).await?;
        } else {
          panic!("You must supply --os and --arch for multi-arch manifests")
        }
      }
    }
  }
  Ok(())
}
