use std::convert::{
  TryFrom,
  TryInto,
};
use std::time::Instant;

use futures::future::try_join_all;
use futures::StreamExt;
use once_cell::sync::OnceCell;
use vino_host::{
  Host,
  HostBuilder,
};
use vino_transport::{
  MessageTransport,
  TransportMap,
  TransportStream,
};

static HOST: OnceCell<Host> = OnceCell::new();

fn get_map() -> TransportMap {
  let mut map = TransportMap::new();
  map.insert(
    "markdown",
    MessageTransport::success(&"# Test markdown

    ## Header 1

    Non debitis quia sint quod deleniti aut sit. Voluptatem quis et velit. Voluptatem harum accusantium quia. Aspernatur est ut delectus culpa quibusdam exercitationem culpa non. Ipsum aliquam ullam …

    ```shell
    $ Code stuff
    ```

    ## Header 2

    > Quotes

    ## Header 3

    - item
    - item
    - item

    Done
    ".to_string()),
  );
  map
}

async fn request(input: (&Host, TransportMap)) -> TransportStream {
  let (host, data) = input;
  let stream = host.request("render", data, None).await.unwrap();
  stream
}

async fn work() {
  let opts = logger::LoggingOptions {
    // trace: true,
    ..Default::default()
  };
  let _guard = logger::init(&opts.name("markdown-benchmark"));

  let mut host = HostBuilder::try_from("./benches/markdown.vino")
    .unwrap()
    .build();
  host.start().await.unwrap();
  let host = HOST.get_or_init(move || host);
  let num: usize = 1000;
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
  let outputs = try_join_all(futures.into_iter().map(tokio::spawn))
    .await
    .unwrap();
  println!("second round ...");
  let _results = try_join_all(outputs.into_iter().map(|stream| {
    tokio::spawn(async {
      let packets = stream.collect::<Vec<_>>().await;
      assert_eq!(packets.len(), 2);
    })
  }))
  .await
  .unwrap();
  println!("done ...");
  let ms = start.elapsed().as_millis();
  let smaller: u128 = num.try_into().unwrap();
  println!(
    "Took {} ms for {} runs (avg: {}/run)",
    ms,
    num,
    ms / smaller
  );
}

fn main() {
  let rt = tokio::runtime::Runtime::new().unwrap();
  rt.block_on(work())
}
