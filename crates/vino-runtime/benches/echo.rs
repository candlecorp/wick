use std::collections::HashMap;

use criterion::{
  black_box,
  criterion_main,
  Criterion,
};
use lazy_static::lazy_static;
use maplit::hashmap;
#[path = "../tests/runtime_utils/mod.rs"]
mod utils;
use utils::*;
use vino_entity::Entity;
use vino_runtime::network::Network;

lazy_static! {
  pub static ref NETWORK: Network = {
    let rt = actix_rt::System::new();
    let (network, _) = rt
      .block_on(init_network_from_yaml("./manifests/v0/network/echo.yaml"))
      .unwrap();
    network
  };
  pub static ref DATA: Data = hashmap! {
      "input".to_owned() => "1234567890".to_owned(),
  };
  pub static ref ENTITY: Entity = Entity::test("wapc_component");
}

pub type Data = HashMap<String, String>;

pub fn bench_async_service(c: &mut Criterion, name: &str) {
  let rt = actix_rt::System::new();

  // start benchmark loops
  c.bench_function(name, move |b| {
    b.iter_custom(|_iters| {
      let start = std::time::Instant::now();
      let args: (&Network, &Data, Entity) = black_box((&NETWORK, &DATA, ENTITY.to_owned()));
      let result = rt.block_on(run(args));
      println!("{:?}", result);
      // check that at least first request succeeded
      start.elapsed()
    })
  });
}

async fn run(input: (&Network, &Data, Entity)) -> Result<()> {
  let (network, data, entity) = input;
  println!("bef");
  let _result = network.request("echo", entity, data).await?;
  println!("aft");
  Ok(())
}

pub fn service_benches() {
  env_logger::init();
  let mut criterion: Criterion<_> = Criterion::default().configure_from_args();
  bench_async_service(&mut criterion, "echo");
}

criterion_main!(service_benches);
