use std::path::Path;

mod test;
use anyhow::Result;
use flow_component::{panic_callback, Component, ComponentError};
use flow_graph_interpreter::error::{InterpreterError, OperationInvalid, ValidationError};
use flow_graph_interpreter::graph::from_def;
use flow_graph_interpreter::{HandlerMap, Interpreter, NamespaceHandler};
use pretty_assertions::assert_eq;
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + 'a>>;

use tracing::Span;
use wick_interface_types::{ComponentMetadata, ComponentSignature, OperationSignature, Type};
use wick_packet::{Invocation, PacketStream, RuntimeConfig};
fn load<T: AsRef<Path>>(path: T) -> Result<wick_config::config::ComponentConfiguration> {
  Ok(
    wick_config::WickConfiguration::load_from_file_sync(path.as_ref())?
      .finish()?
      .try_component_config()?,
  )
}

struct SignatureTestCollection(ComponentSignature);
impl Component for SignatureTestCollection {
  fn handle(
    &self,
    _invocation: Invocation,
    _config: Option<RuntimeConfig>,
    _callback: std::sync::Arc<flow_component::RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    todo!()
  }

  fn signature(&self) -> &ComponentSignature {
    &self.0
  }
}

fn collections(sig: ComponentSignature) -> HandlerMap {
  HandlerMap::new(vec![NamespaceHandler::new(
    "test",
    Box::new(SignatureTestCollection(sig)),
  )])
  .unwrap()
}

fn interp(path: &str, sig: ComponentSignature) -> std::result::Result<Interpreter, InterpreterError> {
  let network = from_def(&mut load(path).unwrap()).unwrap();

  Interpreter::new(
    network,
    None,
    None,
    Some(collections(sig)),
    panic_callback(),
    &Span::current(),
  )
}

#[test_logger::test(tokio::test)]
async fn test_missing_collections() -> Result<()> {
  let mut manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&mut manifest)?;
  let result: std::result::Result<Interpreter, _> =
    Interpreter::new(network, None, None, None, panic_callback(), &Span::current());
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
      ValidationError::UnknownInput {
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
        .add_input("input", Type::String)
        .add_input("OTHER_IN", Type::String)
        .add_output("output", Type::String)
        .add_output("OTHER_OUT", Type::String),
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
    assert_eq!(e, vec![OperationInvalid::new("test".to_owned(), errors)]);
  } else {
    panic!()
  }

  Ok(())
}
