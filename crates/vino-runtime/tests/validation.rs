#[path = "./runtime_utils/mod.rs"]
mod utils;

use utils::*;

#[test_logger::test(actix_rt::test)]
async fn missing_provider() -> Result<()> {
  let result = init_network_from_yaml("./manifests/v0/validation/missing-provider.yaml").await;

  assert!(result.is_err());
  Ok(())
}
