use std::{sync::Arc, time::Duration};

use vino_lattice::{Lattice, NatsOptions};
use vino_rpc::SharedRpcHandler;

use super::Result;
use crate::options::LatticeOptions;

pub(super) async fn connect_to_lattice(
  opts: &LatticeOptions,
  id: String,
  factory: SharedRpcHandler,
  timeout: Duration,
) -> Result<Arc<Lattice>> {
  info!(
    "Connecting to lattice at {} with timeout of {} ms...",
    opts.address,
    timeout.as_millis()
  );
  let lattice = Lattice::connect(NatsOptions {
    address: opts.address.clone(),
    client_id: id.clone(),
    creds_path: opts.creds_path.clone(),
    token: opts.token.clone(),
    timeout,
  })
  .await?;
  info!("Registering '{}' on lattice...", id);
  lattice.handle_namespace(id, factory).await?;
  Ok(Arc::new(lattice))
}
