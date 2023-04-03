use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BufMut;
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

#[derive(Debug, Clone)]
pub(crate) struct SqlWrapper(pub(crate) TypeWrapper);

impl ToSql for SqlWrapper {
  fn to_sql(
    &self,
    ty: &Type,
    out: &mut tokio_postgres::types::private::BytesMut,
  ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
  where
    Self: Sized,
  {
    let sig = self.0.type_signature();
    match *ty {
      Type::CHAR => {
        if matches!(sig, TypeSignature::U8 | TypeSignature::I8) {
          if let Value::Number(v) = self.0.inner() {
            out.put_u8(v.as_u64().unwrap() as u8);
          }
        }
      }
      Type::TEXT | Type::VARCHAR => {
        if matches!(sig, TypeSignature::String) {
          if let Value::String(v) = self.0.inner() {
            out.put_u32(v.len() as u32);
            out.put_slice(v.as_bytes());
          }
        }
      }
      Type::BOOL => {
        if matches!(sig, TypeSignature::Bool) {
          if let Value::Bool(v) = self.0.inner() {
            out.put_u8(if *v { 1 } else { 0 });
          }
        }
      }
      Type::INT2 => {
        if let Value::Number(v) = self.0.inner() {
          let v = v.as_u64().unwrap();
          if matches!(sig, TypeSignature::U8 | TypeSignature::U16) {
            let num: u16 = v.try_into().unwrap();
            out.put_u16(num);
          } else if matches!(sig, TypeSignature::I8 | TypeSignature::I16) {
            let num: i16 = v.try_into().unwrap();
            num.to_sql(ty, out);
          }
        }
      }
      Type::INT4 => {
        if let Value::Number(v) = self.0.inner() {
          let v = v.as_u64().unwrap();
          if matches!(sig, TypeSignature::U8 | TypeSignature::U16 | TypeSignature::U32) {
            let num: u32 = v.try_into().unwrap();
            num.to_sql(ty, out);
          } else if matches!(sig, TypeSignature::I8 | TypeSignature::I16 | TypeSignature::I32) {
            let num: i64 = v.try_into().unwrap();
            num.to_sql(ty, out);
          }
        }
      }
      Type::INT8 => {
        if let Value::Number(v) = self.0.inner() {
          let v = v.as_u64().unwrap();
          if matches!(
            sig,
            TypeSignature::U8 | TypeSignature::U16 | TypeSignature::U32 | TypeSignature::U64
          ) {
            out.put_u64(v);
          } else if matches!(
            sig,
            TypeSignature::I8 | TypeSignature::I16 | TypeSignature::I32 | TypeSignature::I64
          ) {
            out.put_i64(v as _);
          }
        }
      }
      Type::FLOAT4 => {
        if let Value::Number(v) = self.0.inner() {
          if matches!(sig, TypeSignature::F32) {
            let v = v.as_f64().unwrap();
            out.put_f32(v as f32);
          }
        }
      }
      Type::FLOAT8 => {
        if let Value::Number(v) = self.0.inner() {
          if matches!(sig, TypeSignature::F32 | TypeSignature::F64) {
            let v = v.as_f64().unwrap();
            out.put_f64(v);
          }
        }
      }
      Type::OID => {
        if let Value::Number(v) = self.0.inner() {
          if matches!(sig, TypeSignature::U32) {
            let v = v.as_u64().unwrap();
            out.put_u32(v as u32);
          }
        }
      }

      _ => unimplemented!(),
    }
    Ok(IsNull::No)
  }
  accepts! {VARCHAR, OID, FLOAT4, FLOAT8, INT2, INT4, INT8, BOOL, CHAR}

  to_sql_checked! {}
}
