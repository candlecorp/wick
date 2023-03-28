use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use flow_expression_parser::parse::{NS_LINK, SCHEMATIC_OUTPUT, SENDER_ID, SENDER_PORT};
use serde_json::Value;
use tracing::debug;
use wick_config::component_config::{ComponentImplementation, CompositeComponentConfiguration};
use wick_config::error::ManifestError;
use wick_config::*;

fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
  let path = PathBuf::from(path);
  WickConfiguration::load_from_file(path)
}

fn load_component(path: &str) -> Result<CompositeComponentConfiguration, ManifestError> {
  Ok(load(path)?.try_component_config()?.try_composite()?.clone())
}

#[test_logger::test]
fn test_basics() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/logger.yaml")?;

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test]
fn load_minimal() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/minimal.yaml");

  assert!(manifest.is_ok());

  Ok(())
}

#[test_logger::test]
fn load_noversion_yaml() -> Result<(), ManifestError> {
  let result = load("./tests/manifests/v0/noversion.yaml");
  println!("result: {:?}", result);
  assert!(matches!(result, Err(ManifestError::NoFormat)));
  Ok(())
}

#[test_logger::test]
fn load_bad_manifest_yaml() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/bad-yaml.yaml");
  if let Err(Error::YamlError(p, e)) = manifest {
    debug!("{}, {:?}", p, e);
  } else {
    panic!("Should have failed with YamlError but got : {:?}", manifest);
  }

  Ok(())
}

#[test_logger::test]
fn load_collections_yaml() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/collections.yaml")?.try_component_config()?;

  assert_eq!(manifest.name(), &Some("collections".to_owned()));
  if let ComponentImplementation::Composite(component) = manifest.component() {
    assert_eq!(component.components().len(), 4);
  } else {
    panic!("Expected a composite component");
  };

  Ok(())
}

#[test_logger::test]
fn load_shortform_yaml() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/logger-shortform.yaml")?;

  let first_from = &manifest.flow("logger").unwrap().connections[0].from;
  let first_to = &manifest.flow("logger").unwrap().connections[0].to;
  assert_eq!(first_from, &ConnectionTargetDefinition::new("<input>", "input"));
  assert_eq!(first_to, &ConnectionTargetDefinition::new("logger", "input"));

  Ok(())
}

#[test_logger::test]

fn load_env() -> Result<(), ManifestError> {
  env::set_var("TEST_ENV_VAR", "load_manifest_yaml_with_env");
  let manifest = load_component("./tests/manifests/v0/env.yaml")?;

  assert_eq!(
    manifest.flow("name_load_manifest_yaml_with_env").unwrap().name,
    "name_load_manifest_yaml_with_env"
  );

  Ok(())
}

#[test_logger::test]
fn load_sender_yaml() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/sender.yaml")?;

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
  let manifest = load_component("./tests/manifests/v0/ns.yaml")?;

  let schematic = &manifest.flow("logger").unwrap();
  let from = &schematic.connections[0].from;
  assert!(from.matches_instance(NS_LINK));

  Ok(())
}
