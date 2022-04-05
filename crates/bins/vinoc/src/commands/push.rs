use std::path::PathBuf;

use clap::Args;
use nkeys::KeyPairType;
use oci_distribution::client::{ImageData, ImageLayer};
use oci_distribution::manifest;
use oci_distribution::secrets::RegistryAuth;
use vino_wascap::ClaimsOptions;

use crate::error::ControlError;
use crate::io::async_read;
use crate::keys::{extract_keypair, GenerateCommon};
use crate::Result;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  /// OCI reference to push to.
  pub(crate) reference: String,

  /// OCI artifact to push.
  pub(crate) source: PathBuf,

  /// Use --bundle to indicate this is a multi-architecture bundle manifest.
  #[clap(short = 'B', long = "bundle")]
  pub(crate) bundle: bool,

  #[clap(short, long = "rev")]
  pub(crate) rev: Option<u32>,

  #[clap(short, long = "ver")]
  pub(crate) ver: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::oci::Options,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Push artifact");
  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(opts.oci_opts.insecure_registries.clone());
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut client = oci_distribution::Client::new(config);

  let auth = match (opts.oci_opts.username, opts.oci_opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username, password),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };
  if opts.bundle {
    info!("Pushing multi-architecture bundle...");

    let subject_kp = extract_keypair(
      Some(opts.source.to_string_lossy().to_string()),
      opts.common.directory.clone(),
      KeyPairType::Module,
    )
    .await?;

    let issuer_kp = extract_keypair(
      Some(opts.source.to_string_lossy().to_string()),
      opts.common.directory.clone(),
      KeyPairType::Account,
    )
    .await?;

    let archmap = vino_oci::generate_archmap(
      &opts.source,
      ClaimsOptions {
        revision: opts.rev,
        version: opts.ver,
        expires_in_days: opts.common.expires_in_days,
        not_before_days: opts.common.not_before,
      },
      &subject_kp,
      &issuer_kp,
    )
    .await?;

    let reference = vino_oci::parse_reference(&opts.reference)?;

    vino_oci::push_multi_arch(&mut client, &auth, &reference, archmap).await?;
  } else {
    info!("Pushing artifact...");
    let image_ref = vino_oci::parse_reference(&opts.reference)?;
    let image_bytes = async_read(&opts.source).await?;
    let extension = opts.source.extension().unwrap_or_default().to_str().unwrap_or_default();
    let media_type = match extension {
      "wasm" => manifest::WASM_LAYER_MEDIA_TYPE.to_owned(),
      "tar" => manifest::IMAGE_LAYER_MEDIA_TYPE.to_owned(),
      unknown => return Err(ControlError::UnknownFileType(unknown.to_owned())),
    };

    let image_data = ImageData {
      layers: vec![ImageLayer {
        data: image_bytes,
        media_type,
      }],
      digest: None,
    };

    let response = vino_oci::push(&mut client, &auth, &image_ref, image_data).await?;

    println!("Image URL: {}", response.image_url);
    println!("Manifest URL: {}", response.manifest_url);
    println!("Config URL: {}", response.config_url);
  }

  Ok(())
}
