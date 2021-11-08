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
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  // next version, see: https://github.com/rust-lang/rust-clippy/blob/master/CHANGELOG.md
  // clippy::manual_split_once,
  // clippy::derivable_impls,
  // clippy::needless_option_as_deref,
  // clippy::iter_not_returning_iterator,
  // clippy::same_name_method,
  // clippy::manual_assert,
  // clippy::non_send_fields_in_send_ty,
  // clippy::equatable_if_let,
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
#![allow(
  missing_docs,
  clippy::too_many_lines,
  missing_copy_implementations,
  clippy::let_and_return,
  clippy::unused_async
)]

mod generated;

use vino_entity::Entity;
use vino_provider::native::prelude::*;
use vino_random::Random;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcHandler, RpcResult};

use crate::error::NativeError;
use crate::generated::Dispatcher;
mod components;
pub mod error;

#[derive(Clone, Debug)]
pub(crate) struct Context {
  #[allow(unused)]
  rng: Random,
}

impl Context {
  pub(crate) fn new(seed: u64) -> Self {
    let rng = Random::from_seed(seed);
    Self { rng }
  }
}

#[derive(Clone, Debug)]
#[must_use]
pub struct Provider {
  context: Context,
}

impl From<NativeError> for Box<RpcError> {
  fn from(e: NativeError) -> Self {
    Box::new(RpcError::ProviderError(e.to_string()))
  }
}

impl Provider {
  pub fn new(seed: u64) -> Self {
    let context = Context::new(seed);
    Self { context }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    let component = entity.name();
    let result = Dispatcher::dispatch(&component, context, payload).await;
    let stream = result.map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let signature = generated::get_signature();
    Ok(vec![HostedType::Provider(signature)])
  }
}

#[cfg(test)]
mod tests {

  use futures::prelude::*;
  use serde::de::DeserializeOwned;
  use tracing::debug;
  use vino_provider::native::prelude::*;
  use vino_transport::Failure;

  static SEED: u64 = 1000;

  use super::*;
  type Result<T> = std::result::Result<T, NativeError>;

  async fn invoke_one<T>(component: &str, payload: impl Into<TransportMap> + Send) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let transport_map: TransportMap = payload.into();
    println!("TransportMap: {:?}", transport_map);
    let provider = Provider::new(SEED);

    let entity = Entity::component_direct(component);

    let mut outputs = provider.invoke(entity, transport_map).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!(
      "Received payload from port '{}': {:?}",
      output.port, output.payload
    );
    Ok(output.payload.try_into()?)
  }

  async fn invoke_failure(
    component: &str,
    payload: impl Into<TransportMap> + Send,
  ) -> Result<Failure> {
    let transport_map: TransportMap = payload.into();
    println!("TransportMap: {:?}", transport_map);
    let provider = Provider::new(SEED);

    let entity = Entity::component_direct(component);

    let mut outputs = provider.invoke(entity, transport_map).await.unwrap();
    let output = outputs.next().await.unwrap();
    println!(
      "Received payload from port '{}': {:?}",
      output.port, output.payload
    );
    match output.payload {
      MessageTransport::Success(_) => Err("Got success, expected failure".into()),
      MessageTransport::Failure(failure) => Ok(failure),
      MessageTransport::Signal(_) => Err("Got signal, expected failure".into()),
    }
  }

  #[test_logger::test(tokio::test)]
  async fn test_log() -> Result<()> {
    let input = "some_input";
    let job_payload = crate::generated::log::Inputs {
      input: input.to_owned(),
    };
    println!("Inputs: {:?}", job_payload);

    let payload: String = invoke_one("log", job_payload).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, "some_input");

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_gate() -> Result<()> {
    let user_data = "Hello world";
    let exception = "Condition is false";
    let inputs = crate::generated::gate::Inputs {
      condition: true,
      value: MessageTransport::success(&user_data).into(),
      exception: exception.to_owned(),
    };

    let payload: String = invoke_one("gate", inputs).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, user_data);

    let inputs = crate::generated::gate::Inputs {
      condition: false,
      value: MessageTransport::success(&user_data).into(),
      exception: exception.to_owned(),
    };

    let payload = invoke_failure("gate", inputs).await?;

    println!("outputs: {:?}", payload);
    assert_eq!(payload, Failure::Exception(exception.to_owned()));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> Result<()> {
    let provider = Provider::new(SEED);
    let signature = crate::generated::get_signature();
    let components = signature.components.inner();

    let response = provider.get_list().unwrap();

    debug!("list response : {:?}", response);

    assert_eq!(components.len(), 12);
    assert_eq!(response, vec![HostedType::Provider(signature)]);

    Ok(())
  }
}
