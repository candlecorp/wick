use std::path::Path;

mod test;

use anyhow::Result;
use flow_graph_interpreter::error::{InterpreterError, SchematicInvalid, ValidationError};
use flow_graph_interpreter::graph::from_def;
use flow_graph_interpreter::{Component, HandlerMap, Interpreter, NamespaceHandler};
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + Sync + 'a>>;
use seeded_random::Seed;
use serde_json::Value;
use wick_interface_types::{ComponentMetadata, ComponentSignature, OperationSignature, TypeSignature};
use wick_packet::{Invocation, PacketStream};
fn load<T: AsRef<Path>>(path: T) -> Result<wick_config::config::ComponentConfiguration> {
  Ok(wick_config::WickConfiguration::load_from_file(path.as_ref())?.try_component_config()?)
}

struct SignatureTestCollection(ComponentSignature);
impl Component for SignatureTestCollection {
  fn handle(
    &self,
    _invocation: Invocation,
    _stream: PacketStream,
    _config: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, Box<dyn std::error::Error + Send + Sync>>> {
    todo!()
  }

  fn list(&self) -> &ComponentSignature {
    &self.0
  }
}

fn collections(sig: ComponentSignature) -> HandlerMap {
  HandlerMap::new(vec![NamespaceHandler::new(
    "test",
    Box::new(SignatureTestCollection(sig)),
  )])
}

fn interp(path: &str, sig: ComponentSignature) -> std::result::Result<Interpreter, InterpreterError> {
  let network = from_def(&load(path).unwrap()).unwrap();

  Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(collections(sig)))
}

#[test_logger::test(tokio::test)]
async fn test_missing_collections() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest)?;
  let result: std::result::Result<Interpreter, _> = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, None);
  let validation_errors = ValidationError::ComponentIdNotFound("test".to_owned());
  if let Err(InterpreterError::EarlyError(e)) = result {
    assert_eq!(e, validation_errors);
  } else {
    panic!()
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_component() -> Result<()> {
  let result = interp("./tests/manifests/v0/external.yaml", ComponentSignature::default());
  let validation_errors = ValidationError::MissingOperation {
    component: "test".to_owned(),
    name: "echo".to_owned(),
  };
  if let Err(InterpreterError::EarlyError(e)) = result {
    assert_eq!(e, validation_errors);
  } else {
    panic!()
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_invalid_port() -> Result<()> {
  let signature = ComponentSignature::new("instance")
    .version("0.0.0")
    .metadata(ComponentMetadata::default())
    .add_operation(OperationSignature::new("echo"));

  let result = interp("./tests/manifests/v0/external.yaml", signature);

  if let Err(InterpreterError::EarlyError(e)) = result {
    assert_eq!(
      e,
      ValidationError::MissingConnection {
        port: "input".to_owned(),
        operation: "echo".to_owned(),
        component: "test".to_owned(),
      }
    );
  } else {
    panic!()
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_port() -> Result<()> {
  let signature = ComponentSignature::new("test")
    .version("0.0.0")
    .metadata(ComponentMetadata::default())
    .add_operation(
      OperationSignature::new("echo")
        .add_input("input", TypeSignature::String)
        .add_input("OTHER_IN", TypeSignature::String)
        .add_output("output", TypeSignature::String)
        .add_output("OTHER_OUT", TypeSignature::String),
    );

  let result = interp("./tests/manifests/v0/external.yaml", signature);

  let errors = vec![
    ValidationError::MissingConnection {
      port: "OTHER_IN".to_owned(),
      component: "test".to_owned(),
      operation: "echo".to_owned(),
    },
    ValidationError::UnusedOutput {
      port: "OTHER_OUT".to_owned(),
      component: "test".to_owned(),
      operation: "echo".to_owned(),
    },
  ];

  if let Err(InterpreterError::ValidationError(e)) = result {
    assert_eq!(e, vec![SchematicInvalid::new("test".to_owned(), errors)]);
  } else {
    panic!()
  }

  Ok(())
}
