use std::collections::{
  HashSet,
  VecDeque,
};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use vino_component::v0::Payload;
use vino_component::Packet;
use vino_provider::OutputSignal;
use vino_transport::{
  MessageTransportStream,
  TransportWrapper,
};
use vino_types::signatures::ComponentSignature;
use vino_wascap::{
  Claims,
  ComponentClaims,
};
use wapc::WapcHost;

use crate::wapc_module::WapcModule;
use crate::{
  Error,
  Result,
};

type PortBuffer = VecDeque<(String, Packet)>;

#[derive(Debug)]
pub struct WasmHost {
  host: WapcHost,
  claims: Claims<ComponentClaims>,
  buffer: Arc<Mutex<PortBuffer>>,
  closed_ports: Arc<Mutex<HashSet<String>>>,
}

impl TryFrom<&WapcModule> for WasmHost {
  type Error = Error;

  fn try_from(module: &WapcModule) -> Result<Self> {
    let jwt = &module.token.jwt;

    // Ensure that the JWT we found on this actor is valid, not expired, can be used,
    // has a verified signature, etc.
    vino_wascap::validate_token::<ComponentClaims>(jwt).map_err(Error::ClaimsError)?;

    let time = Instant::now();
    vino_macros::mark!();
    #[cfg(feature = "wasmtime")]
    #[allow(unused)]
    let engine = {
      let engine = wasmtime_provider::WasmtimeEngineProvider::new(&module.bytes, None);
      trace!(
        "PRV:WASM:Wasmtime thread loaded in {} μs",
        time.elapsed().as_micros()
      );
      engine
    };
    #[cfg(feature = "wasm3")]
    #[allow(unused)]
    let engine = {
      let engine = wasm3_provider::Wasm3EngineProvider::new(&module.bytes);
      trace!(
        "PRV:WASM:wasm3 thread loaded in {} μs",
        time.elapsed().as_micros()
      );
      engine
    };
    vino_macros::elapsed!();

    let engine = Box::new(engine);
    let buffer = Arc::new(Mutex::new(PortBuffer::new()));
    let inner_buffer = buffer.clone();
    let closed_ports = Arc::new(Mutex::new(HashSet::new()));
    let closed_inner = closed_ports.clone();

    vino_macros::elapsed!();
    let host = WapcHost::new(engine, move |_id, _inv_id, port, output_signal, payload| {
      trace!("PRV:WASM:WAPC_CALLBACK:{:?}", payload);
      match OutputSignal::from_str(output_signal) {
        Ok(signal) => match signal {
          OutputSignal::Output => {
            if closed_inner.lock().contains(port) {
              Err("Closed".into())
            } else {
              inner_buffer
                .lock()
                .push_back((port.to_owned(), payload.into()));
              Ok(vec![])
            }
          }
          OutputSignal::OutputDone => {
            if closed_inner.lock().contains(port) {
              Err("Closed".into())
            } else {
              inner_buffer
                .lock()
                .push_back((port.to_owned(), payload.into()));
              inner_buffer
                .lock()
                .push_back((port.to_owned(), Packet::V0(Payload::Done)));
              closed_inner.lock().insert(port.to_owned());
              Ok(vec![])
            }
          }
          OutputSignal::Done => {
            inner_buffer
              .lock()
              .push_back((port.to_owned(), Packet::V0(Payload::Done)));
            closed_inner.lock().insert(port.to_owned());
            Ok(vec![])
          }
        },
        Err(_) => Err("Invalid signal".into()),
      }
    })?;
    vino_macros::elapsed!();
    info!(
      "Wasmtime thread initialized in {} μs",
      time.elapsed().as_micros()
    );
    Ok(Self {
      claims: module.claims(),
      host,
      buffer,
      closed_ports,
    })
  }
}

impl WasmHost {
  pub fn call(&mut self, component_name: &str, payload: &[u8]) -> Result<MessageTransportStream> {
    {
      self.buffer.lock().clear();
      self.closed_ports.lock().clear();
    }
    trace!("PRV:WASM:INVOKE:{}:START", component_name);
    let _result = self.host.call(component_name, payload)?;
    trace!("PRV:WASM:INVOKE:{}:FINISH", component_name);
    let (tx, rx) = unbounded_channel();
    let mut locked = self.buffer.lock();
    while let Some((port, payload)) = locked.pop_front() {
      let transport = TransportWrapper {
        port,
        payload: payload.into(),
      };
      tx.send(transport).map_err(|_| Error::SendError)?;
    }

    Ok(MessageTransportStream::new(rx))
  }

  pub fn get_components(&self) -> &Vec<ComponentSignature> {
    let claims = &self.claims;
    let components = &claims.metadata.as_ref().unwrap().interface.components;
    components
  }
}
