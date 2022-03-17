use std::path::Path;

use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_interpreter::graph::from_def;
use vino_interpreter::{BoxError, Interpreter, Provider, ProviderNamespace, Providers};
use vino_manifest::Loadable;
use vino_transport::{Invocation, MessageTransport, TransportMap, TransportStream, TransportWrapper};
use vino_types::ProviderSignature;
struct SignatureProvider(ProviderSignature);
impl Provider for SignatureProvider {
  fn handle(&self, _payload: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    todo!()
  }

  fn list(&self) -> &vino_types::ProviderSignature {
    &self.0
  }
}

struct TestProvider(ProviderSignature);
impl TestProvider {
  fn new() -> Self {
    let sig = serde_json::from_value(json!({
      "name":"test-provider",
        "components" : {
          "echo": {
            "name": "echo",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
        }
    }))
    .unwrap();
    Self(sig)
  }
}
impl Provider for TestProvider {
  fn handle(&self, mut invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    let operation = invocation.target.name();
    println!("got op {} in echo test provider", operation);
    let stream = match operation {
      "echo" => {
        let input = invocation.payload.consume_raw("input").unwrap();
        let messages = vec![TransportWrapper::new("output", input), TransportWrapper::done("output")];
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
        Ok(stream)
      }
      _ => Err("Error".into()),
    };
    Box::pin(async move { stream })
  }

  fn list(&self) -> &vino_types::ProviderSignature {
    &self.0
  }
}

fn load<T: AsRef<Path>>(path: T) -> Result<vino_manifest::HostManifest> {
  Ok(vino_manifest::HostManifest::load_from_file(path.as_ref())?)
}

#[test_logger::test(tokio::test)]
async fn test_invoke_provider() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let entity = Entity::component("test", "echo");

  let invocation = Invocation::new_test("invoke provider", entity, inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  let result: String = wrapper.deserialize()?;

  assert_eq!(result, "Hello world".to_owned());
  interpreter.shutdown().await?;

  Ok(())
}
