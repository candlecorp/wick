pub(crate) mod sql_wrapper;

use url::Url;
use wick_config::config::{Metadata, UrlResource};
use wick_config::Resolver;
use wick_interface_types::{component, ComponentSignature, Field, OperationSignature, Type};
use wick_packet::{Packet, TypeWrapper};

use self::sql_wrapper::SqlWrapper;
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

pub(crate) fn convert_url_resource(resolver: &Resolver, id: &str) -> Result<Url> {
  let addr = resolver(id)
    .ok_or_else(|| Error::ResourceNotFound(id.to_owned()))?
    .and_then(|r| r.try_resource())
    .map_err(Error::InvalidResource)?;

  let resource: UrlResource = addr.into();
  resource.url().value().cloned().ok_or(Error::InvalidResourceConfig)
}

pub(crate) fn bind_args(positional_args: &[String], values: &[(Type, Packet)]) -> Result<Vec<SqlWrapper>> {
  let mut bound_args: Vec<SqlWrapper> = Vec::new();
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
            .map(|v| SqlWrapper::from(TypeWrapper::new(*ty.clone(), v))),
        );
      } else {
        return Err(Error::Prepare(format!("value for '{}...' is not an array ", arg)));
      }
    } else {
      bound_args.push(SqlWrapper::from(type_wrapper));
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
    assert_eq!(bound_args[0].clone().decode::<String>().unwrap(), "value1");
    assert_eq!(bound_args[1].clone().decode::<String>().unwrap(), "value2");
    assert_eq!(bound_args[2].clone().decode::<String>().unwrap(), "value3");

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
    assert_eq!(bound_args[0].clone().decode::<String>().unwrap(), "value1");
    assert_eq!(*bound_args[0].0.type_signature(), Type::String);
    assert_eq!(bound_args[1].clone().decode::<String>().unwrap(), "value2.1");
    assert_eq!(*bound_args[1].0.type_signature(), Type::String);
    assert_eq!(bound_args[2].clone().decode::<String>().unwrap(), "value2.2");
    assert_eq!(*bound_args[2].0.type_signature(), Type::String);
    assert_eq!(bound_args[3].clone().decode::<String>().unwrap(), "value3");
    assert_eq!(*bound_args[3].0.type_signature(), Type::String);

    Ok(())
  }
}
