use std::path::Path;

use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::json;
use tokio_stream::StreamExt;
use vino_interpreter::{
  BoxError,
  Interpreter,
  InterpreterError,
  Provider,
  ProviderNamespace,
  Providers,
  ValidationError,
};
use vino_manifest::Loadable;
use vino_schematic_graph::{ExternalReference, Network, Schematic};
use vino_transport::{MessageTransport, TransportMap, TransportStream};
use vino_types::ComponentMap;
struct MyProvider(ComponentMap);
impl Provider for MyProvider {
  fn handle(&self, _operation: &str, _payload: TransportMap) -> BoxFuture<Result<TransportStream, BoxError>> {
    todo!()
  }

  fn list(&self) -> &vino_types::ComponentMap {
    &self.0
  }
}
fn load<T: AsRef<Path>>(path: T) -> Result<vino_manifest::HostManifest> {
  Ok(vino_manifest::HostManifest::load_from_file(path.as_ref())?)
}

fn from_manifest(network_manifest: &vino_manifest::NetworkDefinition) -> Result<Network> {
  let mut network = Network::new(network_manifest.name.clone().unwrap_or_default());

  for m in &network_manifest.schematics {
    let mut schematic = Schematic::new(m.name.clone());

    for (name, def) in m.instances.iter() {
      schematic.add_or_get_instance(name, ExternalReference::new(&def.namespace, &def.name));
    }

    for connection in &m.connections {
      println!("{}", connection);
      let from = &connection.from;
      let to = &connection.to;
      let from_port = if let Some(component) = schematic.find_mut(from.get_instance()) {
        println!("{:?}", component);
        component.add_output(from.get_port())
      } else {
        panic!();
      };
      let to_port = if let Some(component) = schematic.find_mut(to.get_instance()) {
        println!("{:?}", component);
        component.add_input(to.get_port())
      } else {
        panic!();
      };

      schematic.connect(from_port, to_port)?;
    }
    network.add_schematic(schematic);
  }
  Ok(network)
}

#[test_logger::test(tokio::test)]
async fn test_missing_providers() -> Result<()> {
  let manifest = load("./tests/manifests/v0/provider.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, None);
  let validation_errors = vec![ValidationError::MissingProvider("test".to_owned())];
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::ValidationError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_component() -> Result<()> {
  let manifest = load("./tests/manifests/v0/provider.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;

  let sig = ComponentMap::default();
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(MyProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));
  let validation_errors = vec![ValidationError::MissingComponent {
    name: "instance".to_owned(),
    namespace: "test".to_owned(),
  }];
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::ValidationError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_invalid_port() -> Result<()> {
  let manifest = load("./tests/manifests/v0/provider.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;

  let sig = serde_json::from_value(json!({
    "instance" : {
      "name": "instance",
      "inputs": {},
      "outputs": {}
    }
  }))
  .unwrap();
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(MyProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));
  let validation_errors = vec![
    ValidationError::InvalidPort {
      port: "input".to_owned(),
      component: "instance".to_owned(),
      namespace: "test".to_owned(),
    },
    ValidationError::InvalidPort {
      port: "output".to_owned(),
      component: "instance".to_owned(),
      namespace: "test".to_owned(),
    },
  ];
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::ValidationError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_missing_port() -> Result<()> {
  let manifest = load("./tests/manifests/v0/provider.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;

  let sig = serde_json::from_value(json!({
    "instance" : {
      "name": "instance",
      "inputs": {
        "input": {"type":"string"},
        "OTHER_IN": {"type":"string"},
      },
      "outputs": {
        "output": {"type":"string"},
        "OTHER_OUT": {"type":"string"},
      }
    }
  }))
  .unwrap();
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(MyProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));
  let validation_errors = vec![
    ValidationError::MissingPort {
      port: "OTHER_IN".to_owned(),
      component: "instance".to_owned(),
      namespace: "test".to_owned(),
    },
    ValidationError::MissingPort {
      port: "OTHER_OUT".to_owned(),
      component: "instance".to_owned(),
      namespace: "test".to_owned(),
    },
  ];
  if let Err(e) = result {
    assert_eq!(e, InterpreterError::ValidationError(validation_errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_echo() -> Result<()> {
  let manifest = load("./tests/manifests/v0/echo.yaml")?;
  let network = from_manifest(&manifest.network().try_into()?)?;
  let mut interpreter = Interpreter::new(network, None)?;
  interpreter.start().await;
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);
  let schematic = interpreter.schematic("echo").unwrap();
  let stream = schematic.start(Some(inputs)).await?;
  let mut outputs: Vec<_> = stream.collect().await;
  let wrapper = outputs.pop().unwrap();
  println!("packet: {:?}", wrapper);
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  println!("packet: {:?}", wrapper);
  let result: String = wrapper.deserialize()?;

  assert_eq!(result, "Hello world".to_owned());
  interpreter.shutdown().await?;

  Ok(())
}
