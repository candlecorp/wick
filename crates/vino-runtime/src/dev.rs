use std::iter::FromIterator;

use futures::TryFuture;

use crate::dev::prelude::*;
pub(crate) mod prelude {
  pub(crate) use std::convert::TryFrom;

  pub(crate) use actix::prelude::{
    Actor,
    ActorContext,
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
  pub(crate) use tracing::Instrument;
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
  pub(crate) use vino_transport::message_transport::MessageSignal;
  pub(crate) use vino_wascap::{
    ComponentClaims,
    KeyPair,
  };

  pub(crate) use crate::dev::*;
  pub(crate) use crate::dispatch::{
    get_uuid,
    inv_error,
    OutputPacket,
    PortReference,
  };
  pub(crate) use crate::error::{
    InternalError,
    *,
  };
  pub(crate) use crate::models::component_model::ComponentModel;
  pub(crate) use crate::models::*;
  pub(crate) use crate::network_service::NetworkService;
  pub(crate) use crate::prelude::*;
  pub(crate) use crate::schematic_service::SchematicService;
  pub(crate) use crate::utils::actix::ActorResult;
}

pub(crate) trait SendableTryFuture: TryFuture + Send {}

#[allow(clippy::future_not_send)]
pub(crate) async fn join_or_err<I>(
  i: I,
  error_num: i32,
) -> Result<Vec<<<I as IntoIterator>::Item as TryFuture>::Ok>, InternalError>
where
  I: IntoIterator,
  I::Item: TryFuture,
{
  use futures::future::try_join_all;
  Ok(
    try_join_all(i)
      .await
      .map_err(|_| InternalError(error_num))?,
  )
}

pub(crate) fn filter_map<A, B, F>(source: Vec<A>, f: F) -> Vec<B>
where
  A: Sized,
  B: Sized,
  F: FnMut(A) -> Option<B>,
{
  source.into_iter().filter_map(f).collect()
}

pub(crate) fn map<A, B, F, T>(source: &[A], f: F) -> T
where
  A: Sized,
  B: Sized,
  F: FnMut(&A) -> B,
  T: FromIterator<B>,
{
  source.iter().map(f).collect()
}

pub(crate) fn map_into<A, B, F>(source: Vec<A>, f: F) -> Vec<B>
where
  A: Sized,
  B: Sized,
  F: FnMut(A) -> B,
{
  source.into_iter().map(f).collect()
}
