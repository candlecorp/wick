use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use parking_lot::RwLock;
use wasmflow_sdk::v1::codec::messagepack::{deserialize, serialize};
use wasmflow_sdk::v1::packet::Packet;
use wasmflow_sdk::v1::runtime::{LogLevel, OutputSignal};
use wasmflow_sdk::v1::BoxedFuture;

use crate::collection::HostLinkCallback;
use crate::transaction::Transaction;

type InvocationFn =
  dyn Fn(&str, &str, &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> + 'static + Sync + Send;

type AsyncInvocationFn =
  dyn Fn(&str, &str, &[u8]) -> BoxedFuture<Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync;

pub(crate) fn create_log_handler() -> Arc<InvocationFn> {
  Arc::new(move |level: &str, msg: &str, _: &[u8]| {
    match LogLevel::from_str(level) {
      Ok(lvl) => match lvl {
        LogLevel::Info => info!("WASM: {}", msg),
        LogLevel::Error => error!("WASM: {}", msg),
        LogLevel::Warn => warn!("WASM: {}", msg),
        LogLevel::Debug => debug!("WASM: {}", msg),
        LogLevel::Trace => trace!("WASM: {}", msg),
        LogLevel::Mark => {
          let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
          trace!("WASM:[{}]: {}", now.as_millis(), msg);
        }
      },
      Err(_) => {
        return Err(format!("Invalid log level: {}", level).into());
      }
    };
    Ok(vec![])
  })
}

pub(crate) fn create_link_handler(callback: Arc<Option<Box<HostLinkCallback>>>) -> Arc<AsyncInvocationFn> {
  Arc::new(move |origin: &str, target: &str, payload: &[u8]| {
    let origin = origin.to_owned();
    let target = target.to_owned();
    let payload = payload.to_vec();
    let callback = callback.clone();
    Box::pin(async move {
      match callback.as_ref() {
        Some(cb) => {
          trace!(%origin, %target, "wasm link call");
          let now = Instant::now();
          let result = (cb)(
            &origin,
            &target,
            deserialize::<wasmflow_sdk::v1::packet::PacketMap>(&payload)?,
          )
          .await;
          let micros = now.elapsed().as_micros();
          trace!(%origin, %target, durasion_us = %micros, ?result, "wasm link call result");

          match result {
            Ok(packets) => {
              // ensure all packets are messagepack-ed
              let packets: Vec<_> = packets
                .into_iter()
                .map(|mut p| {
                  p.payload.to_messagepack();
                  p
                })
                .collect();
              trace!(%origin, %target, ?payload, "wasm link call payload");
              Ok(serialize(&packets)?)
            }
            Err(e) => Err(e.into()),
          }
        }
        None => Err("Host link called with no callback provided in the WaPC host.".into()),
      }
    })
  })
}

pub(crate) fn create_output_handler(tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>>) -> Arc<InvocationFn> {
  Arc::new(move |port: &str, output_signal, bytes: &[u8]| {
    let payload = &bytes[4..bytes.len()];
    let mut be_bytes: [u8; 4] = [0; 4];
    be_bytes.copy_from_slice(&bytes[0..4]);
    let id: u32 = u32::from_be_bytes(be_bytes);
    trace!(id, port, ?payload, "output payload");
    let mut lock = tx_map.write();
    let mut tx = lock
      .get_mut(&id)
      .ok_or(format!("Invalid transaction (TX: {})", id))?
      .write();

    match OutputSignal::from_str(output_signal) {
      Ok(signal) => {
        if matches!(signal, OutputSignal::Output | OutputSignal::OutputDone) {
          if tx.ports.contains(port) {
            return Err(format!("Port '{}' already closed for (TX: {})", port, id).into());
          }
          let packet = Packet::from_messagepack(payload);
          trace!(id, port, ?packet, "deserialized packet");
          tx.buffer.push_back((port.to_owned(), packet));
        }

        if matches!(signal, OutputSignal::OutputDone | OutputSignal::Done) {
          tx.buffer.push_back((port.to_owned(), Packet::done()));
          trace!(id, port, "port closing");
          tx.ports.insert(port.to_owned());
        }
        Ok(vec![])
      }
      Err(_) => Err("Invalid signal".into()),
    }
  })
}
