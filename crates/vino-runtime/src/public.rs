use std::path::Path;

use crate::components::vino_component::WapcComponent;
use crate::dev::prelude::*;
pub use crate::dispatch::{
  ComponentEntity,
  PortReference,
  VinoEntity,
};
pub use crate::network_provider::Provider as NetworkProvider;

pub fn load_wasm_from_file(path: &Path) -> Result<WapcComponent> {
  Ok(WapcComponent::from_file(path)?)
}

pub async fn load_wasm_from_oci(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let actor_bytes =
    crate::util::oci::fetch_oci_bytes(actor_ref, allow_latest, &allowed_insecure).await?;
  Ok(WapcComponent::from_slice(&actor_bytes)?)
}

pub async fn load_wasm(
  actor_ref: &str,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<WapcComponent> {
  let path = Path::new(&actor_ref);
  if path.exists() {
    Ok(WapcComponent::from_file(path)?)
  } else {
    load_wasm_from_oci(actor_ref, allow_latest, allowed_insecure).await
  }
}
