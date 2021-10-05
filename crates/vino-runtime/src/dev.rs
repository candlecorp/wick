use std::iter::FromIterator;

use futures::TryFuture;

pub(crate) mod prelude {
  pub(crate) use std::convert::TryFrom;

  pub(crate) use actix::prelude::{
    Actor,
    ActorFutureExt,
    Addr,
    Arbiter,
    AsyncContext,
    Context,
    Handler,
    MailboxError,
    Message,
    Recipient,
    ResponseActFuture,
    Supervised,
    System,
    SystemService,
    WrapFuture,
  };
  pub(crate) use futures::FutureExt;
  pub(crate) use itertools::*;
  pub(crate) use tokio_stream::StreamExt;
  pub(crate) use vino_entity::entity::Entity;
  pub(crate) use vino_manifest::{
    parse_id,
    ComponentDefinition,
    ConnectionDefinition,
    ConnectionTargetDefinition,
    ProviderDefinition,
    ProviderKind,
    SchematicDefinition,
  };
  pub(crate) use vino_transport::message_transport::stream::BoxedTransportStream;
  pub(crate) use vino_transport::{
    Failure,
    Invocation,
    MessageSignal,
    MessageTransport,
    Success,
    TransportMap,
    TransportWrapper,
  };
  pub(crate) use vino_types::signatures::*;
  pub(crate) use vino_wascap::KeyPair;

  pub(crate) use crate::core_data::InitData;
  pub(crate) use crate::dev::*;
  pub(crate) use crate::dispatch::{
    inv_error,
    InvocationMessage,
  };
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
    create_network_provider_model,
    initialize_native_provider,
    start_network_provider,
    ProviderChannel,
  };
  pub(crate) use crate::schematic_service::SchematicService;
  pub(crate) use crate::transaction::TransactionUpdate;
  pub(crate) use crate::utils::actix::ActorResult;
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
  source
    .iter()
    .map(|a| a.to_string())
    .collect::<Vec<_>>()
    .join(", ")
}
