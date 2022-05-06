use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{manifest, Client, Reference};
use vino_oci::error::OciError;

use crate::io::async_write;
use crate::Result;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  /// OCI reference to pull.
  pub(crate) reference: String,

  /// Directory to store the pulled artifacts.
  #[clap(short, long = "output", default_value = ".")]
  pub(crate) output: PathBuf,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,

  /// The architecture to pull for multi-architecture artifacts.
  #[clap(long, env = "OCI_ARCH")]
  pub(crate) arch: Option<String>,

  /// The os to pull for multi-architecture artifacts.
  #[clap(long, env = "OCI_ARCH")]
  pub(crate) os: Option<String>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Pull artifact");
  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(opts.oci_opts.insecure_registries.clone());
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut client = Client::new(config);

  let auth = match (&opts.oci_opts.username, &opts.oci_opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username.clone(), password.clone()),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };

  let reference =
    Reference::from_str(&opts.reference).map_err(|e| OciError::OCIParseError(opts.reference.clone(), e.to_string()))?;

  pull(&reference, &opts, &mut client, &auth).await?;
  Ok(())
}

#[async_recursion::async_recursion]
async fn pull(reference: &Reference, opts: &Options, client: &mut Client, auth: &RegistryAuth) -> Result<()> {
  let (manifest, _) = client
    .pull_manifest(reference, auth)
    .await
    .map_err(|e| OciError::OciDistribution(e))?;
  let imagedata = client
    .pull(
      reference,
      auth,
      vec![
        manifest::WASM_LAYER_MEDIA_TYPE,
        manifest::IMAGE_LAYER_MEDIA_TYPE,
        manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE,
        manifest::IMAGE_DOCKER_LAYER_TAR_MEDIA_TYPE,
        manifest::IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE,
      ],
    )
    .await
    .map_err(OciError::OciDistribution)?;

  for (i, layer) in imagedata.layers.into_iter().enumerate() {
    match &manifest {
      oci_distribution::manifest::OciManifest::Image(manifest) => {
        let path = opts.output.clone();
        if let Some(Some(annotations)) = manifest.layers.get(i).map(|l| &l.annotations) {
          if let Some(name) = annotations.get("org.opencontainers.image.title") {
            async_write(path.join(name), layer.data).await?;
          }
        } else {
          async_write(path.join(format!("layer-{}.out", i)), layer.data).await?;
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
