use std::path::Path;

pub mod native_component_actor;
pub mod vino_component;
pub(crate) mod wapc_component_actor;

pub(crate) type Inputs = Vec<String>;
pub(crate) type Outputs = Vec<String>;

use crate::Result;

use self::vino_component::WapcComponent;

pub fn load_wasm_from_file(path: impl AsRef<Path>) -> Result<WapcComponent> {
  WapcComponent::from_file(path)
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
