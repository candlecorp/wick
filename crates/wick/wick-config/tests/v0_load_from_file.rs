use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use flow_expression_parser::parse::{NS_LINK, SCHEMATIC_OUTPUT, SENDER_ID, SENDER_PORT};
use serde_json::{json, Value};
use tracing::debug;
use wick_config::error::ManifestError;
use wick_config::*;

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test]
fn load_minimal() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/minimal.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  assert_eq!(manifest.format(), 0);

  Ok(())
}

#[test_logger::test]
fn load_noversion_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/noversion.yaml");
  let result = ComponentConfiguration::load_from_file(path);
  println!("result: {:?}", result);
  assert!(matches!(result, Err(ManifestError::NoFormat)));
  Ok(())
}

#[test_logger::test]
fn load_bad_manifest_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/bad-yaml.yaml");
  let manifest = ComponentConfiguration::load_from_file(path);
  if let Err(Error::YamlError(e)) = manifest {
    debug!("{:?}", e);
  } else {
    panic!("Should have failed with YamlError but got : {:?}", manifest);
  }

  Ok(())
}

#[test_logger::test]
fn load_collections_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/collections.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  assert_eq!(manifest.name(), &Some("collections".to_owned()));
  assert_eq!(manifest.components().len(), 6);
  assert_eq!(
    manifest.component("wapc2").unwrap().config().unwrap(),
    &json!({"obj":{"data_prop":"data_value"}})
  );

  Ok(())
}

#[test_logger::test]
fn load_shortform_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger-shortform.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  let first_from = &manifest.flow("logger").unwrap().connections[0].from;
  let first_to = &manifest.flow("logger").unwrap().connections[0].to;
  assert_eq!(first_from, &ConnectionTargetDefinition::new("<input>", "input"));
  assert_eq!(first_to, &ConnectionTargetDefinition::new("logger", "input"));

  Ok(())
}

#[test_logger::test]

fn load_env() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/env.yaml");
  env::set_var("TEST_ENV_VAR", "load_manifest_yaml_with_env");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  assert_eq!(
    manifest.flow("name_load_manifest_yaml_with_env").unwrap().name,
    "name_load_manifest_yaml_with_env"
  );

  Ok(())
}

#[test_logger::test]
fn load_sender_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/sender.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  let first_from = &manifest.flow("sender").unwrap().connections[0].from;
  let first_to = &manifest.flow("sender").unwrap().connections[0].to;
  assert_eq!(
    first_from,
    &ConnectionTargetDefinition::new_with_data(SENDER_ID, SENDER_PORT, Value::from_str(r#""1234512345""#).unwrap())
  );
  assert_eq!(first_to, &ConnectionTargetDefinition::new(SCHEMATIC_OUTPUT, "output"));

  Ok(())
}

#[test_logger::test]
fn load_ns_link() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/ns.yaml");
  let manifest = ComponentConfiguration::load_from_file(path)?;

  let schematic = &manifest.flow("logger").unwrap();
  let from = &schematic.connections[0].from;
  assert!(from.matches_instance(NS_LINK));

  Ok(())
}
