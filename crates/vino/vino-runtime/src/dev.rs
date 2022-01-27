use std::iter::FromIterator;

use futures::TryFuture;

pub(crate) mod prelude {
  pub(crate) use std::convert::TryFrom;

  #[allow(unused)]
  pub(crate) use futures::FutureExt;
  pub(crate) use itertools::*;
  pub(crate) use tokio_stream::StreamExt;
  pub(crate) use vino_entity::Entity;
  pub(crate) use vino_manifest::{
    parse_id, ComponentDefinition, ConnectionDefinition, ConnectionTargetDefinition, ProviderDefinition, ProviderKind,
    SchematicDefinition,
  };
  pub(crate) use vino_transport::BoxedTransportStream;
  pub(crate) use vino_transport::{
    Failure, Invocation, MessageSignal, MessageTransport, Success, TransportMap, TransportWrapper,
  };
  pub(crate) use vino_types::*;
  pub(crate) use vino_wascap::KeyPair;

  pub(crate) use crate::dev::*;
  pub(crate) use crate::dispatch::init_data::InitData;
  pub(crate) use crate::dispatch::InvocationMessage;
  pub(crate) use crate::error::*;
  pub(crate) use crate::models::component_model::*;
  pub(crate) use crate::models::network_model::*;
  pub(crate) use crate::models::provider_model::*;
  pub(crate) use crate::models::schematic_model::*;
  pub(crate) use crate::models::*;
  pub(crate) use crate::network_service::NetworkService;
  pub(crate) use crate::prelude::*;
  pub(crate) use crate::providers::network_provider::Provider as NetworkProvider;
  pub(crate) use crate::providers::{
    initialize_native_provider, start_network_provider, BoxedInvocationHandler, InvocationHandler, ProviderChannel,
  };
  pub(crate) use crate::schematic_service::SchematicService;
  pub(crate) use crate::transaction::TransactionUpdate;
  pub(crate) use crate::utils::helpers::*;
}

pub(crate) trait SendableTryFuture: TryFuture + Send {}

pub(crate) fn map<A, B, F, T>(source: &[A], f: F) -> T
where
  A: Sized,
  B: Sized,
  F: FnMut(&A) -> B,
  T: FromIterator<B>,
{
  source.iter().map(f).collect()
}

pub(crate) fn join_comma<A>(source: &[A]) -> String
where
  A: Sized + std::fmt::Display,
{
  source.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
}
