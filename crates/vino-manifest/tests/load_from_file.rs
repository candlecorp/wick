use std::env;
use std::path::PathBuf;

use tracing::debug;
use vino_manifest::error::ManifestError;
use vino_manifest::parse::{NS_LINK, SCHEMATIC_OUTPUT, SENDER_ID, SENDER_PORT};
use vino_manifest::HostDefinition;
use vino_manifest::*;

#[test_logger::test]
fn load_manifest_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");

  Ok(())
}

#[test_logger::test]
fn load_minimal() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/minimal.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.version, 0);

  Ok(())
}

#[test_logger::test]
fn load_noversion_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/noversion.yaml");
  let result = HostManifest::load_from_file(&path);
  println!("result: {:?}", result);
  assert!(matches!(result, Err(ManifestError::NoVersion)));
  Ok(())
}

#[test_logger::test]
fn load_bad_manifest_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/bad-yaml.yaml");
  let manifest = HostManifest::load_from_file(&path);
  if let Err(Error::YamlError(e)) = manifest {
    debug!("{:?}", e);
  } else {
    panic!("Should have failed with YamlError but got : {:?}", manifest);
  }

  Ok(())
}

#[test_logger::test]
fn load_providers_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/providers.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.network.name, Some("providers".to_owned()));
  assert_eq!(manifest.network.providers.len(), 5);

  Ok(())
}

#[test_logger::test]
fn load_shortform_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger-shortform.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");
  let first_from = &manifest.network.schematics[0].connections[0].from;
  let first_to = &manifest.network.schematics[0].connections[0].to;
  assert_eq!(
    first_from,
    &v0::ConnectionTargetDefinition {
      instance: "<input>".to_owned(),
      port: "input".to_owned(),
      data: None,
    }
  );
  assert_eq!(
    first_to,
    &v0::ConnectionTargetDefinition {
      instance: "logger".to_owned(),
      port: "input".to_owned(),
      data: None,
    }
  );

  Ok(())
}

#[test_logger::test]

fn load_env() -> Result<(), ManifestError> {
  println!("Loading yaml");
  let path = PathBuf::from("./tests/manifests/v0/env.yaml");
  env::set_var("TEST_ENV_VAR", "load_manifest_yaml_with_env");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(
    manifest.network.schematics[0].name,
    "name_load_manifest_yaml_with_env"
  );

  Ok(())
}

#[test_logger::test]
fn load_sender_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/sender.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  let first_from = &manifest.network.schematics[0].connections[0].from;
  let first_to = &manifest.network.schematics[0].connections[0].to;
  assert_eq!(
    first_from,
    &v0::ConnectionTargetDefinition {
      instance: SENDER_ID.to_owned(),
      port: SENDER_PORT.to_owned(),
      data: Some(r#""1234512345""#.to_owned()),
    }
  );
  assert_eq!(
    first_to,
    &v0::ConnectionTargetDefinition {
      instance: SCHEMATIC_OUTPUT.to_owned(),
      port: "output".to_owned(),
      data: None,
    }
  );

  Ok(())
}

#[test_logger::test]
fn load_ns_link() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/ns.yaml");
  let manifest = HostDefinition::load_from_file(&path)?;

  let schematic = &manifest.network.schematics[0];
  let from = &schematic.connections[0].from;
  assert!(from.matches_instance(NS_LINK));
  assert!(from.is_sender());

  Ok(())
}
