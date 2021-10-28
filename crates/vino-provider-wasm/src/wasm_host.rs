use std::collections::{
  HashMap,
  HashSet,
  VecDeque,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{
  Instant,
  SystemTime,
};

use parking_lot::RwLock;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_packet::v0::Payload;
use vino_packet::Packet;
use vino_provider::{
  HostCommand,
  LogLevel,
  OutputSignal,
};
use vino_transport::{
  TransportMap,
  TransportStream,
  TransportWrapper,
};
use vino_types::signatures::ProviderSignature;
use vino_wascap::{
  Claims,
  ProviderClaims,
};
use wapc::{
  WapcHost,
  WasiParams,
};

use crate::error::WasmProviderError;
use crate::provider::HostLinkCallback;
use crate::wapc_module::WapcModule;
use crate::{
  Error,
  Result,
};

type PortBuffer = VecDeque<(String, Packet)>;

type InvocationFn = dyn Fn(&str, &str, &[u8]) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
  + 'static
  + Sync
  + Send;

#[must_use]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Box<HostLinkCallback>>,
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
    WasmHost::try_load(module, self.wasi_params, self.callback)
  }
}

impl Default for WasmHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[derive()]
pub struct WasmHost {
  host: RwLock<WapcHost>,
  claims: Claims<ProviderClaims>,
  tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>>,
  rng: vino_random::Random,
}

impl Clone for WasmHost {
  fn clone(&self) -> Self {
    Self {
      host: RwLock::new(self.host.read().clone()),
      claims: self.claims.clone(),
      tx_map: self.tx_map.clone(),
      rng: self.rng.clone(),
    }
  }
}

#[derive(Debug)]
struct Transaction {
  buffer: PortBuffer,
  ports: HashSet<String>,
}

impl Default for Transaction {
  fn default() -> Self {
    Self {
      ports: HashSet::new(),
      buffer: VecDeque::new(),
    }
  }
}

impl std::fmt::Debug for WasmHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHost")
      .field("claims", &self.claims)
      .field("tx_map", &self.tx_map)
      .finish()
  }
}

#[allow(clippy::too_many_lines)]
impl WasmHost {
  pub fn try_load(
    module: &WapcModule,
    wasi_options: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;

    vino_wascap::validate_token::<ProviderClaims>(jwt)
      .map_err(|e| Error::ClaimsInvalid(e.to_string()))?;

    let time = Instant::now();

    #[cfg(feature = "wasmtime")]
    let engine = {
      let engine = wasmtime_provider::WasmtimeEngineProvider::new_with_cache(
        &module.bytes,
        wasi_options,
        None,
      )
      .map_err(|e| WasmProviderError::EngineFailure(e.to_string()))?;
      trace!(
        "WASM:Wasmtime instance loaded in {} μs",
        time.elapsed().as_micros()
      );
      engine
    };

    let engine = Box::new(engine);
    let tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>> =
      Arc::new(RwLock::new(HashMap::new()));
    let tx_map_inner = tx_map.clone();

    let handle_port_output: Box<InvocationFn> =
      Box::new(move |port: &str, output_signal, bytes: &[u8]| {
        let payload = &bytes[4..bytes.len()];
        let mut be_bytes: [u8; 4] = [0; 4];
        be_bytes.copy_from_slice(&bytes[0..4]);
        let id: u32 = u32::from_be_bytes(be_bytes);
        trace!("WASM:ID[{}]:OUTPUT[{}]:PAYLOAD{:?}", id, port, payload,);
        let mut lock = tx_map_inner.write();
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
                tx.buffer
                  .push_back((port.to_owned(), Packet::V0(Payload::Done)));
                trace!("WASM:ID[{}]:OUTPUT[{}]:CLOSING", id, port);
                tx.ports.insert(port.to_owned());
                Ok(vec![])
              }
            }
            OutputSignal::Done => {
              tx.buffer
                .push_back((port.to_owned(), Packet::V0(Payload::Done)));
              trace!("WASM:ID[{}]:OUTPUT[{}]:CLOSING", id, port);
              tx.ports.insert(port.to_owned());
              Ok(vec![])
            }
          },
          Err(_) => Err("Invalid signal".into()),
        }
      });

    let handle_link_call: Box<InvocationFn> = Box::new(
      move |origin: &str, target: &str, payload: &[u8]| match &callback {
        Some(cb) => {
          trace!("WASM:LINK_CALL:PROVIDER[{}],COMPONENT[{}]", origin, target);
          let now = Instant::now();
          let result = (cb)(origin, target, deserialize::<TransportMap>(payload)?);
          let micros = now.elapsed().as_micros();
          trace!(
            "WASM:LINK_CALL:PROVIDER[{}]:COMPONENT[{}]:RESULT[{} μs]:{:?}",
            origin,
            target,
            micros,
            result
          );

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
              trace!(
                "WASM:LINK_CALL:PROVIDER[{}]:COMPONENT[{}]:PAYLOAD:{:?}",
                origin,
                target,
                packets
              );
              Ok(serialize(&packets)?)
            }
            Err(e) => Err(e.into()),
          }
        }
        None => Err("Host link called with no callback provided in the WaPC host.".into()),
      },
    );

    let handle_log_call: Box<InvocationFn> = Box::new(move |level: &str, msg: &str, _: &[u8]| {
      match LogLevel::from_str(level) {
        Ok(lvl) => match lvl {
          LogLevel::Info => info!("WASM: {}", msg),
          LogLevel::Error => error!("WASM: {}", msg),
          LogLevel::Warn => warn!("WASM: {}", msg),
          LogLevel::Debug => debug!("WASM: {}", msg),
          LogLevel::Trace => trace!("WASM: {}", msg),
          LogLevel::Mark => {
            let now = SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap();
            trace!("WASM:[{}]: {}", now.as_millis(), msg);
          }
        },
        Err(_) => {
          return Err(format!("Invalid log level: {}", level).into());
        }
      };
      Ok(vec![])
    });

    let host = WapcHost::new(engine, move |_id, command, arg1, arg2, payload| {
      trace!(
        "WASM:WAPC_CALLBACK:CMD[{}]:ARG1[{}]:ARG2[{}]:PAYLOAD[{} bytes]",
        command,
        arg1,
        arg2,
        payload.len()
      );

      let now = Instant::now();
      let result = match HostCommand::from_str(command) {
        Ok(HostCommand::Output) => handle_port_output(arg1, arg2, payload),
        Ok(HostCommand::LinkCall) => handle_link_call(arg1, arg2, payload),
        Ok(HostCommand::Log) => handle_log_call(arg1, arg2, payload),
        Err(_) => Err(format!("Invalid command: {}", command).into()),
      };
      trace!(
        "WASM:WAPC_CALLBACK:CMD[{}]:ARG1[{}]:ARG2[{}]:TOOK[{} μs]",
        command,
        arg1,
        arg2,
        now.elapsed().as_micros()
      );
      result
    })?;
    debug!(
      "WASM:Wasmtime initialized in {} μs",
      time.elapsed().as_micros()
    );

    Ok(Self {
      claims: module.claims().clone(),
      host: RwLock::new(host),
      tx_map,
      rng: vino_random::Random::new(),
    })
  }

  fn new_tx(&self) -> u32 {
    let mut id = self.rng.get_u32();
    while self.tx_map.read().contains_key(&id) {
      id = self.rng.get_u32();
    }
    self
      .tx_map
      .write()
      .insert(id, RwLock::new(Transaction::default()));
    id
  }

  fn take_tx(&self, id: u32) -> Result<RwLock<Transaction>> {
    self
      .tx_map
      .write()
      .remove(&id)
      .ok_or(WasmProviderError::TxNotFound)
  }
}

impl RpcProxy for WasmHost {
  fn call(
    &self,
    component_name: &str,
    input_map: &HashMap<String, Vec<u8>>,
  ) -> Result<TransportStream> {
    let id = self.new_tx();

    debug!(
      "WASM:INVOKE[{}]:ID[{}]:PAYLOAD{:?}",
      component_name, id, input_map
    );
    trace!("WASM:INVOKE[{}]:ID[{}]:START", component_name, id);

    let payload = serialize(&(id, &input_map)).map_err(WasmProviderError::CodecError)?;

    let now = Instant::now();
    let host = self.host.write();
    let result = host.call(component_name, &payload);
    drop(host);
    trace!(
      "WASM:INVOKE[{}]:ID[{}]:FINISH[{} μs]",
      component_name,
      id,
      now.elapsed().as_micros()
    );
    debug!(
      "WASM:INVOKE[{}]:ID[{}]:RESULT:{:?}",
      component_name, id, result
    );
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

  fn get_components(&self) -> &ProviderSignature {
    let claims = &self.claims;
    &claims.metadata.as_ref().unwrap().interface
  }
}

pub(crate) trait RpcProxy {
  fn call(
    &self,
    component_name: &str,
    input_map: &HashMap<String, Vec<u8>>,
  ) -> Result<TransportStream>;

  fn get_components(&self) -> &ProviderSignature;
}
