use std::path::Path;

mod test;
use anyhow::Result;
use flow_component::{panic_callback, Component, ComponentError};
use flow_graph_interpreter::error::{InterpreterError, OperationInvalid, ValidationError};
use flow_graph_interpreter::graph::{from_def, GraphError};
use flow_graph_interpreter::{HandlerMap, Interpreter, NamespaceHandler};
use pretty_assertions::assert_eq;
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send + 'a>>;

use tracing::Span;
use wick_interface_types::{ComponentMetadata, ComponentSignature, OperationSignature, Type};
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

async fn load<T: AsRef<Path>>(path: T) -> Result<wick_config::config::ComponentConfiguration> {
  Ok(
    wick_config::WickConfiguration::fetch(path.as_ref(), Default::default())
      .await?
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

async fn interp(path: &str, sig: ComponentSignature) -> std::result::Result<Interpreter, InterpreterError> {
  let components = collections(sig);
  let network = from_def(&mut load(path).await.unwrap(), &components).unwrap();

  Interpreter::new(
    network,
    None,
    Some(components),
    panic_callback(),
    None,
    &Span::current(),
  )
}

#[test_logger::test(tokio::test)]
async fn test_missing_component() -> Result<()> {
  let result = from_def(
    &mut load("./tests/manifests/v0/external.yaml").await?,
    &Default::default(),
  );

  let e = result.unwrap_err();
  assert_eq!(
    e,
    GraphError::MissingOperation {
      component: "test".to_owned(),
      operation: "echo".to_owned(),
      available: vec![]
    }
  );

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_invalid_port() -> Result<()> {
  let signature = ComponentSignature::new_named("instance")
    .set_version("0.0.0")
    .metadata(ComponentMetadata::default())
    .add_operation(OperationSignature::new_named("echo"));

  let result = interp("./tests/manifests/v0/external.yaml", signature).await;

  if let Err(InterpreterError::EarlyError(e)) = result {
    assert_eq!(
      e,
      ValidationError::UnknownInput {
        port: "input".to_owned(),
        id: "INSTANCE".to_owned(),
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
  let signature = ComponentSignature::new_named("test")
    .set_version("0.0.0")
    .metadata(ComponentMetadata::default())
    .add_operation(
      OperationSignature::new_named("echo")
        .add_input("input", Type::String)
        .add_input("OTHER_IN", Type::String)
        .add_output("output", Type::String)
        .add_output("OTHER_OUT", Type::String),
    );

  let result = interp("./tests/manifests/v0/external.yaml", signature).await;

  let errors = vec![ValidationError::MissingConnection {
    port: "OTHER_IN".to_owned(),
    id: "INSTANCE".to_owned(),
    component: "test".to_owned(),
    operation: "echo".to_owned(),
  }];

  if let Err(InterpreterError::ValidationError(e)) = result {
    assert_eq!(e, vec![OperationInvalid::new("test".to_owned(), errors)]);
  } else {
    panic!("{:?}", result);
  }

  Ok(())
}
