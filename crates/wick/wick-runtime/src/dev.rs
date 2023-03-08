pub(crate) mod prelude {
  pub(crate) use futures::FutureExt;
  pub(crate) use tokio_stream::StreamExt;
  pub(crate) use wick_config_component::{CollectionDefinition, CollectionKind};
  pub(crate) use wick_interface_types::*;
  pub(crate) use wick_packet::Entity;

  pub(crate) use crate::collections::InvocationHandler;
  pub(crate) use crate::dispatch::InvocationResponse;
  pub(crate) use crate::error::*;
  pub(crate) use crate::network_service::NetworkService;
  pub(crate) use crate::utils::helpers::*;
}
