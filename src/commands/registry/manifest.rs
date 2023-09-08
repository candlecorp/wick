use std::collections::HashMap;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_oci_utils::OciDescriptor;

use crate::utils::get_auth_for_scope;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// OCI reference to pull.
  #[clap(action)]
  pub(crate) reference: String,

  /// Registry to use (overriding configured registry)
  #[clap(long = "registry", action)]
  pub(crate) registry: Option<String>,

  #[clap(flatten)]
  pub(crate) oci_opts: crate::options::oci::OciOptions,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _enter = span.enter();
  let configured_creds = settings
    .credentials
    .iter()
    .find(|c| opts.reference.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci_opts.username.as_deref(),
    opts.oci_opts.password.as_deref(),
  );

  let mut oci_opts = wick_oci_utils::OciOptions::default();
  oci_opts
    .set_allow_insecure(opts.oci_opts.insecure_registries)
    .set_allow_latest(true)
    .set_username(username)
    .set_password(password);

  span.in_scope(|| debug!(options=?oci_opts, reference= opts.reference, "pulling reference"));

  let (manifest, digest) = wick_oci_utils::fetch_image_manifest(&opts.reference, &oci_opts).await?;

  let manifest_layers = manifest
    .layers
    .iter()
    .enumerate()
    .map(|(i, desc)| format!("  {}:\n{}", i, print_oci_descriptor(desc, 4)))
    .collect::<Vec<_>>()
    .join("\n");

  let text = format!(
    r#"# {}:

Digest: {}
Version: {}
Media Type: {}
Config:
{}
Annotations:
{}
Layers:
{}
"#,
    opts.reference,
    digest,
    manifest.schema_version,
    manifest.media_type.as_deref().unwrap_or_default(),
    print_oci_descriptor(&manifest.config, 2),
    print_annotations(&manifest.annotations, 2),
    manifest_layers,
  );

  span.in_scope(|| debug!(%manifest, reference= opts.reference, "pulled manifest"));
  let json = json!({"manifest":&manifest, "digest":digest});

  Ok(StructuredOutput::new(text, json))
}

fn print_annotations(annotations: &Option<HashMap<String, String>>, indent: u8) -> String {
  if let Some(annotations) = annotations {
    return annotations
      .iter()
      .map(|(key, val)| format!("{}{}: {}", " ".repeat(indent as usize), key, val))
      .collect::<Vec<_>>()
      .join("\n");
  }
  String::new()
}

fn print_oci_descriptor(descriptor: &OciDescriptor, indent: u8) -> String {
  let mut text = vec![format!(
    "{}Media Type: {}",
    " ".repeat(indent as usize),
    descriptor.media_type
  )];
  text.push(format!("{}Digest: {}", " ".repeat(indent as usize), descriptor.digest));
  text.push(format!("{}Size: {}", " ".repeat(indent as usize), descriptor.size));
  if let Some(urls) = &descriptor.urls {
    if !urls.is_empty() {
      text.push(format!("{}URLs: {}", " ".repeat(indent as usize), urls.join(", ")));
    }
  }
  if descriptor.annotations.is_some() {
    text.push(format!(
      "{}Annotations:\n{}",
      " ".repeat(indent as usize),
      print_annotations(&descriptor.annotations, indent + 2)
    ));
  }
  text.join("\n")
}
