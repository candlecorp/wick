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
    // clippy::too_many_lines,
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
    // dead_code,
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
    path_statements ,
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
    // unused,
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
#![allow()]

use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_entity::Entity;
use vino_rpc::{
  BoxedPacketStream,
  DurationStatistics,
  RpcHandler,
  RpcResult,
  Statistics,
};

use crate::error::NativeError;
mod components;
pub mod error;
pub type Result<T> = std::result::Result<T, error::NativeError>;

pub(crate) struct State {}

#[derive(Clone)]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Arc::new(Mutex::new(State {})),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn request(
    &self,
    _inv_id: String,
    entity: Entity,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream> {
    let context = self.context.clone();
    let entity_url = entity.url();
    let component = entity
      .into_component()
      .map_err(|_| NativeError::InvalidEntity(entity_url))?;
    let instance = components::get_component(&component);
    match instance {
      Some(instance) => {
        let future = instance.job_wrapper(context, payload);
        Ok(Box::pin(future.await?))
      }
      None => Err(format!("Could not find component: {}", component).into()),
    }
  }

  async fn list_registered(&self) -> RpcResult<Vec<vino_rpc::HostedType>> {
    let components = components::get_all_components();
    Ok(
      components
        .into_iter()
        .map(vino_rpc::HostedType::Component)
        .collect(),
    )
  }

  async fn report_statistics(&self, id: Option<String>) -> RpcResult<Vec<vino_rpc::Statistics>> {
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
  use vino_codec::messagepack::{
    deserialize,
    serialize,
  };
  use vino_component::{
    v0,
    Packet,
  };
  use vino_rpc::HostedType;

  use super::*;

  #[test_env_log::test(tokio::test)]
  async fn request() -> Result<()> {
    let provider = Provider::default();
    let input = "some_input";
    let invocation_id = "INVOCATION_ID";
    let job_payload = hashmap! {
      "input".to_string() => serialize(input)?,
    };

    let entity = Entity::component("log");

    let mut outputs = provider
      .request(invocation_id.to_string(), entity, job_payload)
      .await
      .expect("request failed");
    let output = outputs.next().await.unwrap();
    println!("Received payload from [{}]", output.port);
    let payload: String = match output.packet {
      Packet::V0(v0::Payload::MessagePack(payload)) => deserialize(&payload)?,
      _ => None,
    }
    .unwrap();

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::default();
    let components = crate::components::get_all_components();

    let response = provider.list_registered().await.expect("request failed");

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

    let response = provider
      .report_statistics(None)
      .await
      .expect("request failed");

    debug!("statistics response : {:?}", response);

    assert_eq!(response.len(), 1);

    Ok(())
  }
}
