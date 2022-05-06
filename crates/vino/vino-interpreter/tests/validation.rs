use std::path::Path;

mod test;

use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use vino_interpreter::graph::from_def;
use vino_interpreter::{
  BoxError,
  HandlerMap,
  Interpreter,
  InterpreterError,
  NamespaceHandler,
  Provider,
  SchematicInvalid,
  ValidationError,
};
use vino_manifest::Loadable;
use vino_random::Seed;
use vino_transport::TransportStream;
use wasmflow_interface::ProviderSignature;
use wasmflow_invocation::Invocation;

fn load<T: AsRef<Path>>(path: T) -> Result<vino_manifest::HostManifest> {
  Ok(vino_manifest::HostManifest::load_from_file(path.as_ref())?)
}
struct SignatureProvider(ProviderSignature);
impl Provider for SignatureProvider {
  fn handle(&self, _payload: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    todo!()
  }

  fn list(&self) -> &wasmflow_interface::ProviderSignature {
    &self.0
  }
}

#[test_logger::test(tokio::test)]
async fn test_missing_providers() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let result: std::result::Result<Interpreter, _> = Interpreter::new(Some(Seed::unsafe_new(1)), network, None, None);
  let validation_errors = ValidationError::MissingProvider("test".to_owned());
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::EarlyError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_component() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;

  let sig = ProviderSignature::default();
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> =
    Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers));
  let validation_errors = ValidationError::MissingComponent {
    namespace: "test".to_owned(),
    name: "echo".to_owned(),
  };
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::EarlyError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_invalid_port() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;

  let sig = serde_json::from_value(json!({
    "name":"instance" ,
    "format":1,
    "version":"",
    "components" : {
      "echo":{
        "name": "echo",
        "inputs": {},
        "outputs": {}
      }
    }
  }))
  .unwrap();
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> =
    Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers));

  if let Err(e) = result {
    assert_eq!(
      e,
      InterpreterError::EarlyError(ValidationError::MissingConnection {
        port: "input".to_owned(),
        component: "echo".to_owned(),
        namespace: "test".to_owned(),
      })
    );
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_port() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;

  let sig = serde_json::from_value(json!({
    "name":"test",
    "format":1,
    "version":"",
    "components" : {
      "echo": {
        "name": "echo",
        "inputs": {
          "input": {"type":"string"},
          "OTHER_IN": {"type":"string"},
        },
        "outputs": {
          "output": {"type":"string"},
          "OTHER_OUT": {"type":"string"},
        }
      }
    }
  }))
  .unwrap();
  let providers = HandlerMap::new(vec![NamespaceHandler::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> =
    Interpreter::new(Some(Seed::unsafe_new(1)), network, None, Some(providers));

  let errors = vec![
    ValidationError::MissingConnection {
      port: "OTHER_IN".to_owned(),
      namespace: "test".to_owned(),
      component: "echo".to_owned(),
    },
    ValidationError::UnusedOutput {
      port: "OTHER_OUT".to_owned(),
      namespace: "test".to_owned(),
      component: "echo".to_owned(),
    },
  ];

  if let Err(e) = result {
    assert_eq!(
      e,
      InterpreterError::ValidationError(vec![SchematicInvalid::new("test".to_owned(), errors)])
    );
  }

  Ok(())
}
