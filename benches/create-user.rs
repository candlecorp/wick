use std::convert::{TryFrom, TryInto};
use std::time::Instant;

use futures::future::try_join_all;
use futures::StreamExt;
use once_cell::sync::{Lazy, OnceCell};
use vino_host::{Host, HostBuilder};
use vino_random::Random;
use vino_transport::{MessageTransport, TransportMap, TransportStream};

static RNG: Lazy<Random> = Lazy::new(vino_random::Random::new);
static HOST: OnceCell<Host> = OnceCell::new();

fn get_map() -> TransportMap {
  let mut map = TransportMap::new();
  map.insert("username", MessageTransport::success(&RNG.get_alphanumeric(15)));
  map.insert("password", MessageTransport::success(&RNG.get_alphanumeric(10)));
  map
}

async fn request(input: (&Host, TransportMap)) -> TransportStream {
  let (host, data) = input;
  let stream = host.request("create-user", data, None).await.unwrap();
  stream
}

async fn work() {
  let opts = logger::LoggingOptions {
    // trace: true,
    ..Default::default()
  };
  let _guard = logger::init(&opts.name("create-user-benchmark"));

  let mut host = HostBuilder::try_from("./benches/create-user.vino").unwrap().build();
  host.start().await.unwrap();
  let host = HOST.get_or_init(move || host);
  let num: usize = 100;
  let mut data = Vec::with_capacity(num);
  for _ in 0..num {
    data.push(get_map());
  }

  let mut futures = vec![];
  let start = Instant::now();
  for (_, map) in data.into_iter().enumerate() {
    // print!("Running {}...", i);
    futures.push(request((host, map)));
    // println!("...done")
  }
  println!("first round ...");
  let outputs = try_join_all(futures.into_iter().map(tokio::spawn)).await.unwrap();
  println!("second round ...");
  let _results = try_join_all(outputs.into_iter().map(|stream| {
    tokio::spawn(async {
      stream.collect::<Vec<_>>().await;
    })
  }))
  .await
  .unwrap();
  println!("done ...");
  let ms = start.elapsed().as_millis();
  let smaller: u128 = num.try_into().unwrap();
  println!("Took {} ms for {} runs (avg: {}/run)", ms, num, ms / smaller);
}

fn main() {
  let rt = tokio::runtime::Runtime::new().unwrap();
  rt.block_on(work())
}
