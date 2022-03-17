use std::path::Path;

use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use tokio_stream::StreamExt;
use vino_entity::Entity;
use vino_interpreter::graph::from_def;
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
use vino_provider::ProviderLink;
use vino_transport::{Failure, Invocation, MessageTransport, TransportMap, TransportStream, TransportWrapper};
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
          "concat": {
            "name": "concat",
            "inputs": {
              "left": {"type":"string"},
              "right": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "ref_to_string": {
            "name": "ref_to_string",
            "inputs": {
              "link": {"type":"link"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "exception": {
            "name": "exception",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "panic": {
            "name": "panic",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "copy": {
            "name": "copy",
            "inputs": {
              "input": {"type":"string"},
              "times": {"type":"u64"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          },
          "reverse": {
            "name": "reverse",
            "inputs": {
              "input": {"type":"string"},
            },
            "outputs": {
              "output": {"type":"string"},
            }
          }
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
      "concat" => {
        let left: String = invocation.payload.consume("left").unwrap();
        let right: String = invocation.payload.consume("right").unwrap();
        let result = format!("{}{}", left, right);
        let messages = vec![
          TransportWrapper::new("output", MessageTransport::success(&result)),
          TransportWrapper::done("output"),
        ];
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
        Ok(stream)
      }
      "ref_to_string" => {
        let link: ProviderLink = invocation.payload.consume("link").unwrap();
        let result = link.to_string();
        let messages = vec![
          TransportWrapper::new("output", MessageTransport::success(&result)),
          TransportWrapper::done("output"),
        ];
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
        Ok(stream)
      }
      "reverse" => {
        println!("Reverse payload {:?}", invocation.payload);
        let input: String = invocation.payload.consume("input").unwrap();
        let messages = vec![
          TransportWrapper::new(
            "output",
            MessageTransport::success(&input.chars().rev().collect::<String>()),
          ),
          TransportWrapper::done("output"),
        ];
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));

        Ok(stream)
      }
      "copy" => {
        println!("Reverse payload {:?}", invocation.payload);
        let input: String = invocation.payload.consume("input").unwrap();
        let times: u64 = invocation.payload.consume("times").unwrap();
        let mut messages = vec![];

        for _ in 0..times {
          messages.push(TransportWrapper::new("output", MessageTransport::success(&input)));
        }

        messages.push(TransportWrapper::done("output"));
        println!("Copy messages: {:#?}", messages);
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));
        Ok(stream)
      }
      "exception" => {
        let input = invocation.payload.consume_raw("input").unwrap();
        println!("test::exception got {}", input);

        let messages = vec![
          TransportWrapper::new(
            "output",
            MessageTransport::Failure(Failure::Exception("test::exception".to_owned())),
          ),
          TransportWrapper::done("output"),
        ];
        let stream = TransportStream::new(tokio_stream::iter(messages.into_iter()));

        Ok(stream)
      }
      "panic" => {
        let input = invocation.payload.consume_raw("input").unwrap();
        println!("test::panic got {}", input);
        panic!();
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
async fn test_missing_providers() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, None);
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
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));
  let validation_errors = ValidationError::MissingComponent {
    name: "echo".to_owned(),
    namespace: "test".to_owned(),
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
    "components" : {
      "echo":{
        "name": "echo",
        "inputs": {},
        "outputs": {}
      }
    }
  }))
  .unwrap();
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));

  if let Err(e) = result {
    assert_eq!(
      e,
      InterpreterError::EarlyError(ValidationError::MissingPort {
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
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(SignatureProvider(sig)))]);

  let result: std::result::Result<Interpreter, _> = Interpreter::new(network, Some(providers));

  let errors = vec![
    ValidationError::MissingPort {
      port: "OTHER_IN".to_owned(),
      namespace: "test".to_owned(),
      component: "echo".to_owned(),
    },
    ValidationError::MissingPort {
      port: "OTHER_OUT".to_owned(),
      namespace: "test".to_owned(),
      component: "echo".to_owned(),
    },
  ];

  if let Err(e) = result {
    assert_eq!(e, InterpreterError::ValidationError(errors));
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_echo() -> Result<()> {
  let manifest = load("./tests/manifests/v0/echo.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;

  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("echo", Entity::schematic("echo"), inputs, None);
  let mut interpreter = Interpreter::new(network, None)?;
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

#[test_logger::test(tokio::test)]
async fn test_external_provider() -> Result<()> {
  let manifest = load("./tests/manifests/v0/external.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("external_provider", Entity::schematic("test"), inputs, None);
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

#[test_logger::test(tokio::test)]
async fn test_self() -> Result<()> {
  let manifest = load("./tests/manifests/v0/reference-self.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("parent_input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("self", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 2);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  let result: String = wrapper.deserialize()?;

  assert_eq!(result, "Hello world".to_owned());
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_senders() -> Result<()> {
  let manifest = load("./tests/manifests/v0/senders.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::default();

  let invocation = Invocation::new_test("senders", Entity::schematic("test"), inputs, None);
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

#[test_logger::test(tokio::test)]
async fn test_exception_default() -> Result<()> {
  let manifest = load("./tests/manifests/v0/exception-default.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("exception-default", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  let result: String = wrapper.deserialize()?;

  assert_eq!(result, "eulav tluafeD".to_owned());

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_exception_nodefault() -> Result<()> {
  let manifest = load("./tests/manifests/v0/exception-nodefault.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("exception-nodefault", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_panic() -> Result<()> {
  let manifest = load("./tests/manifests/v0/panic.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("panic", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Failure(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_inherent() -> Result<()> {
  let manifest = load("./tests/manifests/v0/inherent.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::default();

  let invocation = Invocation::new_test("inherent", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_inherent_nested() -> Result<()> {
  let manifest = load("./tests/manifests/v0/inherent-nested.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::default();

  let invocation = Invocation::new_test("inherent_nested", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_inherent_disconnected() -> Result<()> {
  let manifest = load("./tests/manifests/v0/inherent-disconnected.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);

  let invocation = Invocation::new_test("inherent_disconnected", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 2);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_stream() -> Result<()> {
  let manifest = load("./tests/manifests/v0/stream.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);
  let input_str = "Hello world".to_owned();
  let inputs = TransportMap::from([("input", input_str.clone())]);

  let invocation = Invocation::new_test("stream", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 6);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  for wrapper in outputs {
    let output: String = wrapper.payload.deserialize()?;
    assert_eq!(output, input_str);
  }
  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_spread() -> Result<()> {
  let manifest = load("./tests/manifests/v0/spread.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "Hello world".to_owned())]);
  let invocation = Invocation::new_test("spread", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 4);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  interpreter.shutdown().await?;

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_provider_ref() -> Result<()> {
  let manifest = load("./tests/manifests/v0/provider-ref.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::default();
  let invocation = Invocation::new_test("provider_ref", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 2);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  let wrapper = outputs.pop().unwrap();
  assert!(matches!(wrapper.payload, MessageTransport::Success(_)));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_stream_provider_ref() -> Result<()> {
  let manifest = load("./tests/manifests/v0/stream-provider-ref.yaml")?;
  let network = from_def(&manifest.network().try_into()?)?;
  let providers = Providers::new(vec![ProviderNamespace::new("test", Box::new(TestProvider::new()))]);

  let inputs = TransportMap::from([("input", "my-input".to_owned())]);
  let invocation = Invocation::new_test("stream_provider_ref", Entity::schematic("test"), inputs, None);
  let mut interpreter = Interpreter::new(network, Some(providers))?;
  interpreter.start().await;
  let stream = interpreter.invoke(invocation).await?;

  let mut outputs: Vec<_> = stream.collect().await;
  interpreter.shutdown().await?;
  println!("{:#?}", outputs);
  assert_eq!(outputs.len(), 6);
  let wrapper = outputs.pop().unwrap();
  assert_eq!(wrapper.payload, MessageTransport::done());
  for wrapper in outputs {
    assert!(matches!(wrapper.payload, MessageTransport::Success(_)));
  }

  Ok(())
}
