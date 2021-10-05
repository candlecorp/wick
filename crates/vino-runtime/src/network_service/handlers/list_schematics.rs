use std::collections::HashMap;

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message)]
#[rtype(result = "Result<ProviderSignature>")]
pub(crate) struct GetSignature {}

impl Handler<GetSignature> for NetworkService {
  type Result = Result<ProviderSignature>;

  fn handle(&mut self, _msg: GetSignature, _ctx: &mut Context<Self>) -> Self::Result {
    self.ensure_is_started()?;
    let state = self.state.as_ref().unwrap();
    let resolution_order = {
      let model = state.model.read();
      model
        .get_resolution_order()
        .map_err(|e| NetworkError::UnresolvableNetwork(e.to_string()))?
    };

    trace!(
      "NETWORK:RESOLUTION_ORDER:[{}]",
      join_comma(
        &resolution_order
          .iter()
          .map(|v| format!("[{}]", join_comma(v)))
          .collect::<Vec<_>>()
      )
    );

    let mut signatures = HashMap::new();
    for batch in resolution_order {
      for name in batch {
        trace!("NETWORK:SIGNATURE[{}]:REQUEST", name);
        let schematic_model = { state.model.read().get_schematic(&name).cloned() };

        match schematic_model {
          Some(schematic_model) => {
            let signature = {
              schematic_model
                .read()
                .get_signature()
                .cloned()
                .ok_or_else(|| {
                  NetworkError::UnresolvableNetwork(format!(
                    "Schematic '{}' does not have a signature",
                    name
                  ))
                })?
            };
            let mut scw = state.model.write();
            scw
              .update_self_component(name, signature.clone())
              .map_err(|e| NetworkError::InvalidState(e.to_string()))?;
            signatures.insert(signature.name.clone(), signature);
          }
          None => {
            return Err(NetworkError::InvalidState(format!(
              "Attempted to resolve schematic '{}' but '{}' is not running.",
              name, name
            )));
          }
        }
      }
    }

    let provider_signature = ProviderSignature {
      name: self.uid.clone(),
      components: signatures.into(),
      types: StructMap::new(),
    };

    Ok(provider_signature)
  }
}
