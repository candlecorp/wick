use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::Instant;

use vino_provider::ComponentSignature;
use vino_rpc::port::{
  PortStream,
  Sender,
};
use vino_wascap::{
  Claims,
  ComponentClaims,
};
use wapc::WapcHost;

use crate::output_sender::OutputSender;
use crate::wapc_module::WapcModule;
use crate::{
  Error,
  Result,
};

#[derive(Debug)]
pub struct WasmHost {
  host: WapcHost,
  claims: Claims<ComponentClaims>,
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
      trace!("Wasmtime loaded in {} μs", time.elapsed().as_micros());
      engine
    };
    #[cfg(feature = "wasm3")]
    let engine = {
      let engine = wasm3_provider::Wasm3EngineProvider::new(&module.bytes);
      trace!("wasm3 loaded in {} μs", time.elapsed().as_micros());
      engine
    };

    let engine = Box::new(engine);

    let host = WapcHost::new(engine, move |_, _, _, _, _| Ok(vec![]))?;
    Ok(Self {
      claims: module.claims(),
      host,
    })
  }
}

impl WasmHost {
  pub fn call(&mut self, component_name: &str, payload: &[u8]) -> Result<PortStream> {
    let claims = &self.claims;
    let components = &claims.metadata.as_ref().unwrap().interface.components;

    let component = components
      .iter()
      .find(|c| c.name == component_name)
      .ok_or_else(|| {
        Error::ComponentNotFound(
          component_name.to_owned(),
          components.iter().map(|c| c.name.clone()).collect(),
        )
      })?;

    let senders: HashMap<String, OutputSender> = component
      .outputs
      .iter()
      .map(|port| (port.name.clone(), OutputSender::new(port.name.clone())))
      .collect();

    let ports = senders.iter().map(|(_, o)| o.port.clone()).collect();
    let receiver = PortStream::new(ports);

    self
      .host
      .set_callback(move |_id, inv_id, port, _op, payload| {
        debug!("Payload WaPC host callback: {:?}", payload);

        match senders.get(port) {
          Some(sender) => {
            sender.send_message(payload.into());
          }
          None => {
            error!(
              "Could not get port sender for {} on transaction {}",
              port, inv_id
            );
          }
        }
        Ok(vec![])
      });

    trace!("Calling component {}", component_name);
    let _result = self.host.call(component_name, payload)?;
    debug!("Invocation response: {:?}", _result);

    Ok(receiver)
  }

  pub fn get_components(&self) -> &Vec<ComponentSignature> {
    let claims = &self.claims;
    let components = &claims.metadata.as_ref().unwrap().interface.components;
    components
  }
}
