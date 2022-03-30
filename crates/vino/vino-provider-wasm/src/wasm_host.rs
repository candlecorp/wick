use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use parking_lot::RwLock;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_codec::messagepack::{deserialize, serialize};
use vino_packet::v0::Payload;
use vino_packet::Packet;
use vino_transport::{TransportMap, TransportStream, TransportWrapper};
use vino_types::ProviderSignature;
use vino_wapc::{HostCommand, LogLevel, OutputSignal};
use vino_wascap::{Claims, ProviderClaims};
use wapc::{WapcHost, WasiParams};
use wapc_pool::{HostPool, HostPoolBuilder};

use crate::error::WasmProviderError;
use crate::provider::HostLinkCallback;
use crate::wapc_module::WapcModule;
use crate::{Error, Result};

type PortBuffer = VecDeque<(String, Packet)>;

type InvocationFn = dyn Fn(&str, &str, &[u8]) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
  + 'static
  + Sync
  + Send;

#[must_use]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Box<HostLinkCallback>>,
  min_threads: usize,
  max_threads: usize,
}

impl std::fmt::Debug for WasmHostBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHostBuilder")
      .field("wasi_params", &self.wasi_params)
      .finish()
  }
}

impl WasmHostBuilder {
  pub fn new() -> Self {
    Self {
      wasi_params: None,
      callback: None,
      min_threads: 1,
      max_threads: 1,
    }
  }

  pub fn wasi_params(mut self, params: WasiParams) -> Self {
    self.wasi_params = Some(params);
    self
  }

  pub fn link_callback(mut self, callback: Box<HostLinkCallback>) -> Self {
    self.callback = Some(callback);
    self
  }

  pub fn preopened_dirs(mut self, dirs: Vec<String>) -> Self {
    let mut params = self.wasi_params.take().unwrap_or_default();
    params.preopened_dirs = dirs;
    self.wasi_params.replace(params);
    self
  }

  pub fn build(self, module: &WapcModule) -> Result<WasmHost> {
    WasmHost::try_load(
      module,
      self.wasi_params,
      self.callback,
      self.min_threads,
      self.max_threads,
    )
  }

  pub fn max_threads(mut self, max_threads: usize) -> Self {
    self.max_threads = max_threads;
    self
  }

  pub fn min_threads(mut self, min_threads: usize) -> Self {
    self.min_threads = min_threads;
    self
  }
}

impl Default for WasmHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[derive()]
pub struct WasmHost {
  host: HostPool,
  claims: Claims<ProviderClaims>,
  tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>>,
  rng: vino_random::Random,
}

#[derive(Debug, Default)]
struct Transaction {
  buffer: PortBuffer,
  ports: HashSet<String>,
}

impl std::fmt::Debug for WasmHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHost")
      .field("claims", &self.claims)
      .field("tx_map", &self.tx_map)
      .finish()
  }
}

impl WasmHost {
  pub fn try_load(
    module: &WapcModule,
    wasi_options: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
    min_threads: usize,
    max_threads: usize,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;

    vino_wascap::validate_token::<ProviderClaims>(jwt).map_err(|e| Error::ClaimsInvalid(e.to_string()))?;

    let time = Instant::now();

    let tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>> = Arc::new(RwLock::new(HashMap::new()));
    let link_callback = Arc::new(callback);

    #[cfg(feature = "wasmtime")]
    let engine = {
      let engine = wasmtime_provider::WasmtimeEngineProvider::new_with_cache(&module.bytes, wasi_options, None)
        .map_err(|e| WasmProviderError::EngineFailure(e.to_string()))?;
      trace!(duration_nanos = ?time.elapsed().as_micros(), "wasmtime instance loaded");
      engine
    };

    let engine = Box::new(engine);

    let tx_map_inner = tx_map.clone();

    let pool = HostPoolBuilder::new()
      .name("wasmtime-test")
      .factory(move || {
        let handle_port_output = create_output_handler(tx_map_inner.clone());
        let handle_link_call = create_link_handler(link_callback.clone());
        let handle_log_call = create_log_handler();

        let host_callback: Box<wapc::HostCallback> = Box::new(move |_id, command, arg1, arg2, payload| {
          trace!(command, arg1, arg2, len = payload.len(), "wapc callback");

          let now = Instant::now();
          let result = match HostCommand::from_str(command) {
            Ok(HostCommand::Output) => handle_port_output(arg1, arg2, payload),
            Ok(HostCommand::LinkCall) => handle_link_call(arg1, arg2, payload),
            Ok(HostCommand::Log) => handle_log_call(arg1, arg2, payload),
            Err(_) => Err(format!("Invalid command: {}", command).into()),
          };
          trace!(
            command, arg1, arg2, duration_nanos = ?now.elapsed().as_micros(),
            "wapc callback done",
          );
          result
        });

        WapcHost::new(engine.clone(), Some(host_callback)).unwrap()
      })
      .min_threads(min_threads)
      .max_threads(max_threads)
      .build();

    debug!(duration_nanos = ?time.elapsed().as_micros(), "wasmtime initialize");

    Ok(Self {
      claims: module.claims().clone(),
      host: pool,
      tx_map,
      rng: vino_random::Random::new(),
    })
  }

  fn new_tx(&self) -> u32 {
    let mut id = self.rng.get_u32();
    while self.tx_map.read().contains_key(&id) {
      id = self.rng.get_u32();
    }
    self.tx_map.write().insert(id, RwLock::new(Transaction::default()));
    id
  }

  fn take_tx(&self, id: u32) -> Result<RwLock<Transaction>> {
    self.tx_map.write().remove(&id).ok_or(WasmProviderError::TxNotFound)
  }

  pub async fn call(&self, component_name: &str, input_map: &HashMap<String, Vec<u8>>) -> Result<TransportStream> {
    let id = self.new_tx();

    debug!(component = component_name, id, payload = ?input_map, "wasm invoke");

    let payload = serialize(&(id, &input_map)).map_err(WasmProviderError::CodecError)?;

    let now = Instant::now();
    let result = self.host.call(component_name, payload).await;
    trace!(
      component = component_name,
      id,
      duration_nanos = ?now.elapsed().as_micros(),
      "wasm call finished"
    );
    trace!(component = component_name, id, ?result, "wasm call result");
    let transaction = self.take_tx(id)?;
    if let Err(e) = result {
      return Err(e.into());
    };
    let (tx, rx) = unbounded_channel();
    let mut locked = transaction.write();
    while let Some((port, payload)) = locked.buffer.pop_front() {
      let transport = TransportWrapper {
        port,
        payload: payload.into(),
      };
      tx.send(transport).map_err(|_| Error::SendError)?;
    }

    Ok(TransportStream::new(UnboundedReceiverStream::new(rx)))
  }

  pub fn get_components(&self) -> &ProviderSignature {
    let claims = &self.claims;
    &claims.metadata.as_ref().unwrap().interface
  }
}

fn create_log_handler() -> Box<InvocationFn> {
  Box::new(move |level: &str, msg: &str, _: &[u8]| {
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

fn create_link_handler(callback: Arc<Option<Box<HostLinkCallback>>>) -> Box<InvocationFn> {
  Box::new(
    move |origin: &str, target: &str, payload: &[u8]| match callback.as_ref() {
      Some(cb) => {
        trace!(origin, target, "wasm link call");
        let now = Instant::now();
        let result = (cb)(origin, target, deserialize::<TransportMap>(payload)?);
        let micros = now.elapsed().as_micros();
        trace!(origin, target, duration_nanos = ?micros, ?result, "wasm link call result");

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
            trace!(origin, target, ?payload, "wasm link call payload");
            Ok(serialize(&packets)?)
          }
          Err(e) => Err(e.into()),
        }
      }
      None => Err("Host link called with no callback provided in the WaPC host.".into()),
    },
  )
}

fn create_output_handler(tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>>) -> Box<InvocationFn> {
  Box::new(move |port: &str, output_signal, bytes: &[u8]| {
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
      Ok(signal) => match signal {
        OutputSignal::Output => {
          if tx.ports.contains(port) {
            Err(format!("Port '{}' already closed for (TX: {})", port, id).into())
          } else {
            tx.buffer.push_back((port.to_owned(), payload.into()));
            Ok(vec![])
          }
        }
        OutputSignal::OutputDone => {
          if tx.ports.contains(port) {
            Err(format!("Port '{}' already closed for (TX: {})", port, id).into())
          } else {
            tx.buffer.push_back((port.to_owned(), payload.into()));
            tx.buffer.push_back((port.to_owned(), Packet::V0(Payload::Done)));
            trace!(id, port, "port closing");
            tx.ports.insert(port.to_owned());
            Ok(vec![])
          }
        }
        OutputSignal::Done => {
          tx.buffer.push_back((port.to_owned(), Packet::V0(Payload::Done)));
          trace!(id, port, "port closing");
          tx.ports.insert(port.to_owned());
          Ok(vec![])
        }
      },
      Err(_) => Err("Invalid signal".into()),
    }
  })
}
