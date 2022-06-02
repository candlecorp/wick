use std::sync::Arc;
use std::time::Duration;

use wasmflow_mesh::{Mesh, NatsOptions};
use wasmflow_rpc::SharedRpcHandler;

use super::Result;
use crate::options::MeshOptions;

pub(super) async fn connect_to_mesh(
  opts: &MeshOptions,
  id: String,
  factory: SharedRpcHandler,
  timeout: Duration,
) -> Result<Arc<Mesh>> {
  info!(
    "Connecting to mesh at {} with timeout of {} ms...",
    opts.address,
    timeout.as_millis()
  );
  let mesh = Mesh::connect(NatsOptions {
    address: opts.address.clone(),
    client_id: id.clone(),
    creds_path: opts.creds_path.clone(),
    token: opts.token.clone(),
    timeout,
  })
  .await?;
  info!("Registering '{}' on mesh...", id);
  mesh.handle_namespace(id, factory).await?;
  Ok(Arc::new(mesh))
}
