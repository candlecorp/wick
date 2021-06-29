use std::path::Path;

pub mod grpc_url_provider;
pub mod native_provider;
pub mod vino_component;
pub(crate) mod wapc_provider;

pub(crate) type Inputs = Vec<String>;
pub(crate) type Outputs = Vec<String>;

use actix::prelude::*;
use vino_rpc::{
  HostedType,
  Statistics,
};

use self::vino_component::WapcComponent;
use crate::{
  Invocation,
  Result,
};

pub fn load_wasm_from_file(path: &Path) -> Result<WapcComponent> {
  WapcComponent::from_file(path)
}

pub async fn load_wasm_from_oci(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let actor_bytes =
    crate::util::oci::fetch_oci_bytes(actor_ref, allow_latest, &allowed_insecure).await?;
  Ok(WapcComponent::from_slice(&actor_bytes)?)
}

pub async fn load_wasm(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let path = Path::new(&actor_ref);
  if path.exists() {
    Ok(WapcComponent::from_file(path)?)
  } else {
    load_wasm_from_oci(actor_ref, allow_latest, allowed_insecure).await
  }
}

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
