pub(crate) mod grpc_provider_service;
pub(crate) mod native_provider_service;
pub(crate) mod network_provider;

use vino_rpc::Statistics;

use crate::dev::prelude::*;
use crate::error::ProviderError;

// This is mostly unused right now except for in tests. The goal was to migrate away
// from actix but that has been put on hold until there are more integration tests.
#[derive(Debug)]
pub(crate) enum ProviderRequest {
  #[allow(unused)]
  Invoke(Invocation),
  #[allow(unused)]
  List(ListRequest),
  #[allow(unused)]
  Statistics(StatsRequest),
}

impl Message for ProviderRequest {
  type Result = Result<ProviderResponse, ProviderError>;
}

#[derive(Debug)]
pub(crate) struct ListRequest {}
#[derive(Debug)]
pub(crate) struct StatsRequest {}

#[derive(Debug)]
pub(crate) enum ProviderResponse {
  List(Vec<HostedType>),
  Stats(Vec<Statistics>),
}

impl ProviderResponse {
  #[allow(unused)]
  pub(crate) fn into_list_response(self) -> Result<Vec<HostedType>, ConversionError> {
    match self {
      ProviderResponse::List(v) => Ok(v),
      _ => Err(ConversionError("Provider response to list")),
    }
  }
  #[allow(unused)]
  pub(crate) fn into_stats_response(self) -> Result<Vec<Statistics>, ConversionError> {
    match self {
      ProviderResponse::Stats(v) => Ok(v),
      _ => Err(ConversionError("Provider response to stats")),
    }
  }
}
