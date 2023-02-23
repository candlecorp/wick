pub(crate) mod prelude {
  pub(crate) use futures::FutureExt;
  pub(crate) use tokio_stream::StreamExt;
  pub(crate) use wasmflow_entity::Entity;
  pub(crate) use wasmflow_interface::*;
  pub(crate) use wasmflow_manifest::{CollectionDefinition, CollectionKind};

  pub(crate) use crate::collections::InvocationHandler;
  pub(crate) use crate::dispatch::InvocationResponse;
  pub(crate) use crate::error::*;
  pub(crate) use crate::network_service::NetworkService;
  pub(crate) use crate::utils::helpers::*;
}
