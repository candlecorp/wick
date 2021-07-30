use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use vino_component::Packet;
use vino_transport::{
  InvocationTransport,
  MessageTransportStream,
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

type PortBuffer = VecDeque<(String, Vec<u8>)>;

#[derive(Debug)]
pub struct WasmHost {
  host: WapcHost,
  claims: Claims<ComponentClaims>,
  buffer: Arc<Mutex<PortBuffer>>,
}

impl TryFrom<&WapcModule> for WasmHost {
  type Error = Error;

  fn try_from(module: &WapcModule) -> Result<Self> {
    let jwt = &module.token.jwt;

    // Ensure that the JWT we found on this actor is valid, not expired, can be used,
    // has a verified signature, etc.
    vino_wascap::validate_token::<ComponentClaims>(jwt)
      .map_err(|e| Error::ClaimsError(e.to_string()))?;

    let time = Instant::now();
    #[cfg(feature = "wasmtime")]
    let engine = {
      let engine = wasmtime_provider::WasmtimeEngineProvider::new(&module.bytes, None);
      trace!(
        "PRV:WASM:Wasmtime loaded in {} μs",
        time.elapsed().as_micros()
      );
      engine
    };
    #[cfg(feature = "wasm3")]
    let engine = {
      let engine = wasm3_provider::Wasm3EngineProvider::new(&module.bytes);
      trace!("PRV:WASM:wasm3 loaded in {} μs", time.elapsed().as_micros());
      engine
    };

    let engine = Box::new(engine);
    let buffer = Arc::new(Mutex::new(VecDeque::new()));
    let inner = buffer.clone();

    let host = WapcHost::new(engine, move |_id, _inv_id, port, _op, payload| {
      trace!("PRV:WASM:WAPC_CALLBACK:{:?}", payload);
      inner.lock().push_back((port.to_owned(), payload.to_vec()));
      Ok(vec![])
    })?;
    Ok(Self {
      claims: module.claims(),
      host,
      buffer,
    })
  }
}

impl WasmHost {
  pub fn call(&mut self, component_name: &str, payload: &[u8]) -> Result<MessageTransportStream> {
    {
      self.buffer.lock().clear();
    }
    trace!("PRV:WASM:INVOKE:{}:START", component_name);
    let _result = self.host.call(component_name, payload)?;
    trace!("PRV:WASM:INVOKE:{}:FINISH", component_name);
    let (tx, rx) = unbounded_channel();
    let mut locked = self.buffer.lock();
    while let Some((port, payload)) = locked.pop_front() {
      let packet: Packet = (&payload).into();
      let transport = InvocationTransport {
        port,
        payload: packet.into(),
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
