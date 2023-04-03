// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
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
#![allow(unused, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]
mod component;
mod conversions;
mod error;
mod from_sql_wrapper;
mod to_sql_wrapper;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BufMut;
pub use component::Component;
pub use error::Error;
use futures::{pin_mut, Future, StreamExt, TryStreamExt};
use openssl::ssl::{SslConnector, SslMethod};
use parking_lot::Mutex;
use postgres_openssl::{MakeTlsConnector, TlsConnector};
use serde_json::{Number, Value};
use tokio::net::{TcpSocket, TcpStream};
use tokio_postgres::tls::{MakeTlsConnect, NoTlsStream, TlsStream};
use tokio_postgres::types::{accepts, to_sql_checked, FromSql, FromSqlOwned, IsNull, ToSql, Type};
use tokio_postgres::{Client, Config, Connection, NoTls, Statement};
use wick_config::config::components::{PostgresComponent, PostgresOperationDefinition};
use wick_config::config::{ConfigurationItem, TcpPort, UdpPort};
use wick_interface_types::{component, ComponentSignature, HostedType, TypeSignature};
use wick_packet::{FluxChannel, Invocation, Observer, Packet, PacketPayload, PacketStream, StreamMap, TypeWrapper};
use wick_rpc::error::RpcError;
use wick_rpc::{dispatch, BoxFuture, RpcHandler, RpcResult};

use crate::conversions::postgres_row_to_json_value;

#[macro_use]
extern crate tracing;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum NativeComponentError {
  #[error("Invalid configuration")]
  InvalidConfig,
  #[error("Temp error")]
  Temp,
  #[error("Invalid output for operations {}. At this time postgres operations can have at most one output named 'output' of type 'object'", .0.join(", "))]
  InvalidOutput(Vec<String>),
}

pub trait NativeComponent {
  type Config;
  fn init(
    &self,
    config: Self::Config,
    app_config: wick_config::config::AppConfiguration,
    // ) -> BoxFuture<Result<(), NativeComponentError>>;
  ) -> Pin<Box<dyn Future<Output = Result<(), NativeComponentError>> + Send>>;
}
