// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::if_then_some_else_none,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs, clippy::too_many_lines)]

pub mod generated;

use std::sync::{
  Arc,
  Mutex,
};

use vino_entity::Entity;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  BoxedTransportStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};

use crate::error::NativeError;
mod components;
pub mod error;
pub type Result<T> = std::result::Result<T, NativeError>;

#[derive(Clone, Debug)]
pub(crate) struct State {}

#[derive(Clone, Debug)]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl From<NativeError> for Box<RpcError> {
  fn from(e: NativeError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

impl Provider {
  #[must_use]
  pub fn default() -> Self {
    Self {
      context: Arc::new(Mutex::new(State {})),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let entity_url = entity.url();
    let component = entity
      .into_component()
      .map_err(|_| NativeError::InvalidEntity(entity_url))?;
    let instance = generated::get_component(&component);
    match instance {
      Some(instance) => {
        let stream = instance
          .execute(context, payload)
          .await
          .map_err(|e| RpcError::ProviderError(e.to_string()))?;
        Ok(Box::pin(stream))
      }
      None => Err(Box::new(RpcError::ProviderError(format!(
        "Component '{}' not found",
        component
      )))),
    }
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = generated::get_all_components();
    Ok(components.into_iter().map(HostedType::Component).collect())
  }

  async fn get_stats(&self, id: Option<String>) -> RpcResult<Vec<Statistics>> {
    // TODO Dummy implementation
    if id.is_some() {
      Ok(vec![Statistics {
        num_calls: 1,
        execution_duration: DurationStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    } else {
      Ok(vec![Statistics {
        num_calls: 0,
        execution_duration: DurationStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    }
  }
}

#[cfg(test)]
mod tests {

  use futures::prelude::*;
  use log::debug;
  use maplit::hashmap;
  use serde::de::DeserializeOwned;
  use vino_provider::native::prelude::*;

  use super::*;

  async fn invoke<T>(component: &str, payload: TransportMap) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let provider = Provider::default();

    let entity = Entity::component(component);

    let mut outputs = provider.invoke(entity, payload).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!("Received payload from [{}]", output.port);
    Ok(output.payload.try_into()?)
  }

  #[test_env_log::test(tokio::test)]
  async fn test_log() -> Result<()> {
    let input = "some_input";
    let job_payload = TransportMap::with_map(hashmap! {
      "input".to_owned() => MessageTransport::messagepack(input),
    });

    let payload: String = invoke("log", job_payload).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_uuid() -> Result<()> {
    let job_payload = TransportMap::new();

    let payload: String = invoke("uuid", job_payload).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload.len(), 36);

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::default();
    let components = crate::generated::get_all_components();

    let response = provider.get_list().await.unwrap();

    debug!("list response : {:?}", response);

    assert_eq!(components.len(), response.len());
    for index in 0..components.len() {
      assert_eq!(
        HostedType::Component(components[index].clone()),
        response[index]
      );
    }

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn statistics() -> Result<()> {
    let provider = Provider::default();

    let response = provider.get_stats(None).await.unwrap();

    debug!("statistics response : {:?}", response);

    assert_eq!(response.len(), 1);

    Ok(())
  }
}
