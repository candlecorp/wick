mod observer;
mod test_component;

use flow_component::panic_callback;
use flow_graph_interpreter::graph::from_def;
use flow_graph_interpreter::Interpreter;
pub use observer::JsonWriter;
use seeded_random::Seed;
pub use test_component::TestComponent;
use wick_packet::{Entity, OperationConfig, Packet};

use crate::test;

#[allow(unused)]
pub(crate) async fn common_setup(
  manifest: &str,
  op: &str,
  packets: Vec<Packet>,
) -> anyhow::Result<(Interpreter, Vec<Result<Packet, wick_packet::Error>>)> {
  base_setup(manifest, Entity::local(op), packets, None).await
}

#[allow(unused)]
pub(crate) async fn base_setup(
  manifest: &str,
  entity: Entity,
  packets: Vec<Packet>,
  config: Option<OperationConfig>,
) -> anyhow::Result<(Interpreter, Vec<Result<Packet, wick_packet::Error>>)> {
  use flow_graph_interpreter::{HandlerMap, InterpreterOptions, NamespaceHandler};
  use tokio_stream::StreamExt;
  use wick_packet::{Entity, Invocation};
  let options = Some(InterpreterOptions {
    error_on_hung: true,
    // TODO: improve logic to ensure no remaining packets are sent after completion.
    // Turn this on to make tests fail in these cases.
    error_on_missing: false,
    ..Default::default()
  });
  let mut def = wick_config::WickConfiguration::load_from_file_sync(manifest)?.try_component_config()?;
  let network = from_def(&mut def)?;
  let collections = HandlerMap::new(vec![NamespaceHandler::new(
    "test",
    Box::new(test::TestComponent::new()),
  )])
  .unwrap();
  let invocation = Invocation::new(Entity::test("test"), entity, None);

  let mut interpreter = Interpreter::new(
    Some(Seed::unsafe_new(1)),
    network,
    None,
    Some(collections),
    panic_callback(),
  )?;
  interpreter.start(options, None).await;
  let stream = wick_packet::PacketStream::new(Box::new(futures::stream::iter(packets.into_iter().map(Ok))));
  let stream = interpreter.invoke(invocation, stream, config).await?;
  let outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  Ok((interpreter, outputs))
}
