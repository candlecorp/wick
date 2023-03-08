// !!START_LINTS
// Wick lints
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
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
  bad_style,
  clashing_extern_declarations,
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
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

use futures::stream::StreamExt;
use wick_config::ComponentConfiguration;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketStream};
use wick_runtime::{Network, NetworkBuilder};

#[macro_use]
extern crate tracing;

pub async fn init_network_from_yaml(path: &str) -> anyhow::Result<(Network, uuid::Uuid)> {
  let host_def = ComponentConfiguration::load_from_file(path)?;
  debug!("Manifest loaded");

  let builder = NetworkBuilder::from_definition(host_def)?;

  let network = builder.build().await?;

  let nuid = network.uid;
  Ok((network, nuid))
}

pub async fn tester(path: &str, stream: PacketStream, target: &str, mut expected: Vec<Packet>) -> anyhow::Result<()> {
  let (network, _) = init_network_from_yaml(path).await?;
  let inherent = InherentData::new(1, 1000);

  let result = network
    .invoke(
      Invocation::new(Entity::test("simple schematic"), Entity::local(target), Some(inherent)),
      stream,
    )
    .await?;

  println!("Result: {:?}", result);
  let messages: Vec<_> = result.collect().await;
  assert_eq!(messages.len(), expected.len());
  expected.reverse();
  for packet in messages {
    let expected = expected.pop().unwrap();
    assert_eq!(packet.unwrap(), expected);
  }

  Ok(())
}
