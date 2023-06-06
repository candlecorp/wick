use std::env;
use std::path::PathBuf;

use flow_expression_parser::ast::{
  ConnectionExpression,
  ConnectionTargetExpression,
  FlowExpression,
  InstancePort,
  InstanceTarget,
};
use tracing::debug;
use wick_config::config::CompositeComponentImplementation;
use wick_config::error::ManifestError;
use wick_config::*;

async fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
  let path = PathBuf::from(path);
  WickConfiguration::load_from_file(path).await
}

async fn load_component(path: &str) -> Result<CompositeComponentImplementation, ManifestError> {
  Ok(load(path).await?.try_component_config()?.try_composite()?.clone())
}

#[test_logger::test(tokio::test)]
async fn test_basics() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/logger.yaml").await?;

  assert_eq!(manifest.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_minimal() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/minimal.yaml").await;

  assert!(manifest.is_ok());

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_noversion_yaml() -> Result<(), ManifestError> {
  let result = load("./tests/manifests/v0/noversion.yaml").await;
  println!("result: {:?}", result);
  assert!(matches!(result, Err(ManifestError::NoFormat(_))));
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_bad_manifest_yaml() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/bad-yaml.yaml").await;
  if let Err(Error::YamlError(p, e, _)) = manifest {
    debug!("{:?}, {:?}", p, e);
  } else {
    panic!("Should have failed with YamlError but got : {:?}", manifest);
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_collections_yaml() -> Result<(), ManifestError> {
  let manifest = load("./tests/manifests/v0/collections.yaml")
    .await?
    .try_component_config()?;

  assert_eq!(manifest.name(), Some(&"collections".to_owned()));
  assert_eq!(manifest.import().len(), 3);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_shortform_yaml() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/logger-shortform.yaml").await?;

  let expr = &manifest.flow("logger").unwrap().expressions()[0];

  assert_eq!(
    expr,
    &FlowExpression::connection(ConnectionExpression::new(
      ConnectionTargetExpression::new(InstanceTarget::Input, InstancePort::named("input")),
      ConnectionTargetExpression::new(InstanceTarget::named("logger"), InstancePort::named("input"))
    ))
  );

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_env() -> Result<(), ManifestError> {
  env::set_var("TEST_ENV_VAR", "load_manifest_yaml_with_env");
  let manifest = load_component("./tests/manifests/v0/env.yaml").await?;

  assert_eq!(
    manifest.flow("name_load_manifest_yaml_with_env").unwrap().name(),
    "name_load_manifest_yaml_with_env"
  );

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn load_ns_link() -> Result<(), ManifestError> {
  let manifest = load_component("./tests/manifests/v0/ns.yaml").await?;

  let schematic = &manifest.flow("logger").unwrap();
  let from = &schematic.expressions()[0].as_connection().unwrap().from();

  assert_eq!(from.instance(), &InstanceTarget::Link);

  Ok(())
}
