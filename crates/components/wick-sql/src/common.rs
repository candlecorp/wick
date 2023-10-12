pub(crate) mod sql_wrapper;

use futures::stream::BoxStream;
use serde_json::Value;
use url::Url;
use wick_config::config::{BoundIdentifier, ErrorBehavior, Metadata, UrlResource};
use wick_config::Resolver;
use wick_interface_types::{component, ComponentSignature, Field, OperationSignature, Type};
use wick_packet::{Packet, PacketExt, TypeWrapper};

use self::sql_wrapper::ConvertedType;
use crate::error::ConversionError;
use crate::Error;
type Result<T> = std::result::Result<T, Error>;

pub(crate) fn gen_signature(
  name: &str,
  operations: Vec<OperationSignature>,
  config: &[Field],
  metadata: &Option<Metadata>,
) -> Result<ComponentSignature> {
  let mut sig = component! {
    name: name,
    version: metadata.as_ref().map(|v|v.version().to_owned()),
    operations: operations,
  };
  sig.config = config.to_vec();

  // NOTE: remove this must change when db components support customized outputs.
  sig.operations.iter_mut().for_each(|op| {
    if !op.outputs.iter().any(|f| f.name == "output") {
      op.outputs.push(Field::new("output", Type::Object));
    }
  });
  Ok(sig)
}

pub(crate) fn convert_url_resource(resolver: &Resolver, id: &BoundIdentifier) -> Result<Url> {
  let addr = resolver(id).and_then(|r| r.try_resource())?;

  let resource: UrlResource = addr.try_into()?;
  resource.url().value().cloned().ok_or(Error::InvalidResourceConfig)
}

pub(crate) fn bind_args(positional_args: &[String], values: &[(Type, Packet)]) -> Result<Vec<ConvertedType>> {
  let mut bound_args: Vec<ConvertedType> = Vec::new();
  for arg in positional_args {
    let (arg, spread) = if arg.ends_with("...") {
      let arg = arg.trim_end_matches("...");
      (arg.trim_end_matches("..."), true)
    } else {
      (arg.as_ref(), false)
    };

    let (ty, packet) = values
      .iter()
      .find(|(_, p)| p.port() == arg)
      .cloned()
      .ok_or_else(|| Error::MissingPacket(arg.to_owned()))?;

    let type_wrapper = packet
      .to_type_wrapper(ty.clone())
      .map_err(|e| Error::Prepare(e.to_string()))?;

    if spread {
      let Type::List { ty } = type_wrapper.type_signature().clone() else {
        return Err(Error::Prepare(format!("value for '{}...' is not a list ", arg)));
      };
      if let serde_json::Value::Array(arr) = type_wrapper.into_inner() {
        bound_args.extend(
          arr
            .into_iter()
            .map(|v| sql_wrapper::convert(&TypeWrapper::new(*ty.clone(), v)))
            .collect::<std::result::Result<Vec<_>, ConversionError>>()
            .map_err(|e| Error::Prepare(e.to_string()))?,
        );
      } else {
        return Err(Error::Prepare(format!("value for '{}...' is not an array ", arg)));
      }
    } else {
      bound_args.push(sql_wrapper::convert(&type_wrapper).map_err(|e| Error::Prepare(e.to_string()))?);
    }
  }

  Ok(bound_args)
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_bound_args() -> Result<()> {
    let bound_args = ["arg1".to_owned(), "arg2".to_owned(), "arg3".to_owned()];
    let values = [
      (Type::String, Packet::encode("arg1", "value1")),
      (Type::String, Packet::encode("arg2", "value2")),
      (Type::String, Packet::encode("arg3", "value3")),
    ];
    let bound_args = bind_args(&bound_args, &values)?;
    assert_eq!(bound_args.len(), 3);
    assert_eq!(bound_args[0], ConvertedType::String(Some("value1".to_owned())));
    assert_eq!(bound_args[1], ConvertedType::String(Some("value2".to_owned())));
    assert_eq!(bound_args[2], ConvertedType::String(Some("value3".to_owned())));

    Ok(())
  }

  #[test]
  fn test_bound_args_spread() -> Result<()> {
    let bound_args = ["arg1".to_owned(), "arg2...".to_owned(), "arg3".to_owned()];
    let values = [
      (Type::String, Packet::encode("arg1", "value1")),
      (
        Type::List {
          ty: Box::new(Type::String),
        },
        Packet::encode("arg2", ["value2.1", "value2.2"]),
      ),
      (Type::String, Packet::encode("arg3", "value3")),
    ];
    let bound_args = bind_args(&bound_args, &values)?;
    assert_eq!(bound_args.len(), 4);
    assert_eq!(bound_args[0], ConvertedType::String(Some("value1".to_owned())));
    assert_eq!(bound_args[1], ConvertedType::String(Some("value2.1".to_owned())));
    assert_eq!(bound_args[2], ConvertedType::String(Some("value2.2".to_owned())));
    assert_eq!(bound_args[3], ConvertedType::String(Some("value3".to_owned())));

    Ok(())
  }
}

#[async_trait::async_trait]
pub(crate) trait DatabaseProvider {
  fn get_statement<'a>(&'a self, id: &'a str) -> Option<&'a str>;
  async fn get_connection<'a, 'b>(&'a self) -> Result<Connection<'b>>
  where
    'a: 'b;
}

#[async_trait::async_trait]
pub(crate) trait ClientConnection: Send + Sync {
  async fn query<'a, 'b>(
    &'a mut self,
    stmt: &'a str,
    bound_args: Vec<ConvertedType>,
  ) -> Result<BoxStream<'b, Result<Value>>>
  where
    'a: 'b;

  async fn exec(&mut self, stmt: String, bound_args: Vec<ConvertedType>) -> Result<u64>;
  async fn finish(&mut self, behavior: ErrorBehavior) -> Result<()>;
  async fn handle_error(&mut self, e: Error, behavior: ErrorBehavior) -> Result<()>;
  async fn start(&mut self, behavior: ErrorBehavior) -> Result<()>;
}

#[derive()]
pub(crate) struct Connection<'a>(Box<dyn ClientConnection + Sync + Send + 'a>);

impl<'conn> Connection<'conn> {
  pub(crate) fn new(conn: Box<dyn ClientConnection + Sync + Send + 'conn>) -> Self {
    Self(conn)
  }

  pub(crate) async fn query<'a, 'b>(
    &'a mut self,
    stmt: &'a str,
    bound_args: Vec<ConvertedType>,
  ) -> Result<BoxStream<'b, Result<Value>>>
  where
    'a: 'b,
  {
    let stream = self.0.query(stmt, bound_args).await?;

    Ok(stream)
  }
  pub(crate) async fn exec(&mut self, stmt: String, bound_args: Vec<ConvertedType>) -> Result<u64> {
    self.0.exec(stmt, bound_args).await
  }

  pub(crate) async fn handle_error(&mut self, e: Error, behavior: ErrorBehavior) -> Result<()> {
    self.0.handle_error(e, behavior).await
  }

  pub(crate) async fn start(&mut self, behavior: ErrorBehavior) -> Result<()> {
    self.0.start(behavior).await
  }

  #[allow(clippy::unused_async)]
  pub(crate) async fn finish(&mut self) -> Result<()> {
    // todo
    Ok(())
  }
}
