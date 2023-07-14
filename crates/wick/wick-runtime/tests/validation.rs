mod utils;

use utils::*;
type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn missing_collection() -> Result<()> {
  let result = init_engine_from_yaml("./tests/manifests/v0/validation/missing-collection.yaml", None).await;
  println!("result: {:?}", result);

  assert!(result.is_err());
  Ok(())
}
