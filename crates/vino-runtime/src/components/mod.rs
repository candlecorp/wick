use std::path::Path;

pub mod grpc_url_provider;
pub mod native_component_actor;
pub mod native_provider;
pub mod provider_component;
pub mod vino_component;
pub(crate) mod wapc_component_actor;

pub(crate) type Inputs = Vec<String>;
pub(crate) type Outputs = Vec<String>;

use actix::prelude::*;
use vino_rpc::{
  HostedType,
  Statistics,
};

use self::native_provider::NativeProvider;
use self::vino_component::WapcComponent;
use crate::actix::ActorResult;
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
// #[rtype(result = "Result<ProviderResponse>")]
pub(crate) enum ProviderMessage {
  Invoke(Invocation),
  List(ListRequest),
  Statistics(StatsRequest),
}

impl Message for ProviderMessage {
  type Result = Result<ProviderResponse>;
}

#[derive(Debug)]
pub(crate) struct ListRequest {}
#[derive(Debug)]
pub(crate) struct StatsRequest {}

#[derive(Debug)]
pub(crate) enum ProviderResponse {
  InvocationResponse,
  ListResponse(Vec<HostedType>),
  StatsResponse(Vec<Statistics>),
}

impl ProviderResponse {
  pub fn into_list_response(self) -> Result<Vec<HostedType>> {
    match self {
      ProviderResponse::ListResponse(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError),
    }
  }
  pub fn into_stats_response(self) -> Result<Vec<Statistics>> {
    match self {
      ProviderResponse::StatsResponse(v) => Ok(v),
      _ => Err(crate::error::VinoError::ConversionError),
    }
  }
  pub fn into_invocation_response(self) -> Result<()> {
    todo!()
  }
}
