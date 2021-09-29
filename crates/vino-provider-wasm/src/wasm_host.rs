use std::collections::{
  HashSet,
  VecDeque,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
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
  OutputSignal,
};
use vino_transport::{
  TransportMap,
  TransportStream,
  TransportWrapper,
};
use vino_types::signatures::ComponentSignature;
use vino_wascap::{
  Claims,
  ComponentClaims,
};
use wapc::{
  WapcHost,
  WasiParams,
};

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
#[allow(missing_debug_implementations)]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Box<HostLinkCallback>>,
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

#[derive(Debug)]
pub struct WasmHost {
  host: WapcHost,
  claims: Claims<ComponentClaims>,
  buffer: Arc<Mutex<PortBuffer>>,
  closed_ports: Arc<Mutex<HashSet<String>>>,
}

impl WasmHost {
  pub fn try_load(
    module: &WapcModule,
    wasi_options: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;

    vino_wascap::validate_token::<ComponentClaims>(jwt).map_err(Error::ClaimsError)?;

    let time = Instant::now();

    let engine = {
      let engine =
        wasmtime_provider::WasmtimeEngineProvider::new(&module.bytes, wasi_options, None).unwrap();
      trace!(
        "WASM:Wasmtime instance loaded in {} μs",
        time.elapsed().as_micros()
      );
      engine
    };

    let engine = Box::new(engine);
    let buffer = Arc::new(Mutex::new(PortBuffer::new()));
    let buffer_inner = buffer.clone();
    let closed_ports = Arc::new(Mutex::new(HashSet::new()));
    let ports_inner = closed_ports.clone();

    let handle_port_output: Box<InvocationFn> =
      Box::new(move |port: &str, output_signal, payload: &[u8]| {
        let mut ports_locked = ports_inner.lock();
        let mut buffer_locked = buffer_inner.lock();

        match OutputSignal::from_str(output_signal) {
          Ok(signal) => match signal {
            OutputSignal::Output => {
              if ports_locked.contains(port) {
                Err("Closed".into())
              } else {
                buffer_locked.push_back((port.to_owned(), payload.into()));
                Ok(vec![])
              }
            }
            OutputSignal::OutputDone => {
              if ports_locked.contains(port) {
                Err("Closed".into())
              } else {
                buffer_locked.push_back((port.to_owned(), payload.into()));
                buffer_locked.push_back((port.to_owned(), Packet::V0(Payload::Done)));
                ports_locked.insert(port.to_owned());
                Ok(vec![])
              }
            }
            OutputSignal::Done => {
              buffer_locked.push_back((port.to_owned(), Packet::V0(Payload::Done)));
              ports_locked.insert(port.to_owned());
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
          let result = (cb)(origin, target, deserialize::<TransportMap>(payload)?);
          match result {
            Ok(packets) => Ok(serialize(&packets)?),
            Err(e) => Err(e.into()),
          }
        }
        None => Err("Host link called with no callback provided in the WaPC host.".into()),
      },
    );

    let host = WapcHost::new(engine, move |_id, command, arg1, arg2, payload| {
      trace!(
        "WASM:WAPC_CALLBACK[CMD={}][arg1={}][arg2={}][PAYLOAD={:?}]",
        command,
        arg1,
        arg2,
        payload
      );

      match HostCommand::from_str(command) {
        Ok(HostCommand::Output) => handle_port_output(arg1, arg2, payload),
        Ok(HostCommand::LinkCall) => handle_link_call(arg1, arg2, payload),
        Err(_) => Err(format!("Invalid command: {}", command).into()),
      }
    })?;
    debug!(
      "WASM:Wasmtime initialized in {} μs",
      time.elapsed().as_micros()
    );

    Ok(Self {
      claims: module.claims().clone(),
      host,
      buffer,
      closed_ports,
    })
  }

  pub fn call(&mut self, component_name: &str, payload: &[u8]) -> Result<TransportStream> {
    {
      self.buffer.lock().clear();
      self.closed_ports.lock().clear();
    }
    trace!("WASM:INVOKE:{}:START", component_name);
    let _result = self.host.call(component_name, payload)?;
    trace!("WASM:INVOKE:{}:FINISH", component_name);
    let (tx, rx) = unbounded_channel();
    let mut locked = self.buffer.lock();
    while let Some((port, payload)) = locked.pop_front() {
      let transport = TransportWrapper {
        port,
        payload: payload.into(),
      };
      tx.send(transport).map_err(|_| Error::SendError)?;
    }

    Ok(TransportStream::new(UnboundedReceiverStream::new(rx)))
  }

  pub fn get_components(&self) -> &Vec<ComponentSignature> {
    let claims = &self.claims;
    let components = &claims.metadata.as_ref().unwrap().interface.components;
    components
  }
}
