pub(crate) mod grpc_url_provider;
pub(crate) mod native_provider;
pub(crate) mod vino_component;
pub(crate) mod wapc_provider;

use vino_rpc::{
  HostedType,
  Statistics,
};

use crate::dev::prelude::*;

#[derive(Debug)]
pub(crate) enum ProviderRequest {
  Invoke(Invocation),
  List(ListRequest),
  Statistics(StatsRequest),
}

impl Message for ProviderRequest {
  type Result = Result<ProviderResponse>;
}

#[derive(Debug)]
pub(crate) struct ListRequest {}
#[derive(Debug)]
pub(crate) struct StatsRequest {}

#[derive(Debug)]
pub(crate) enum ProviderResponse {
  InvocationResponse,
  List(Vec<HostedType>),
  Stats(Vec<Statistics>),
}

impl ProviderResponse {
  pub(crate) fn into_list_response(self) -> Result<Vec<HostedType>> {
    match self {
      ProviderResponse::List(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError(
        "into_list_response",
      )),
    }
  }
  pub(crate) fn into_stats_response(self) -> Result<Vec<Statistics>> {
    match self {
      ProviderResponse::Stats(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError(
        "into_stats_response",
      )),
    }
  }
  pub(crate) fn into_invocation_response(self) -> Result<()> {
    todo!()
  }
}
