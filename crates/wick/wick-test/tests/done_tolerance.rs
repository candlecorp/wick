use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use flow_component::{Component, RuntimeCallback, SharedComponent};
use serde_json::json;
use tokio_stream::StreamExt;
use wick_config::config::{TestCaseBuilder, TestConfigurationBuilder, TestPacket};
use wick_interface_types::{component, ComponentSignature};
use wick_packet::{Invocation, Packet, PacketStream, RuntimeConfig};
use wick_test::{ComponentFactory, TestSuite};

struct TestComponent {
  signature: ComponentSignature,
}

impl TestComponent {
  fn new() -> Self {
    Self {
      signature: component! {
        name: "test",
        version: Some("0.0.1"),
        operations: {
          "echo" => {
            inputs: {
              "in" => "object",
            },
            outputs: {
              "out" => "object",
            },
          },
        }
      },
    }
  }
}

impl Component for TestComponent {
  fn handle(
    &self,
    mut invocation: Invocation,
    _data: Option<RuntimeConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<std::result::Result<PacketStream, flow_component::ComponentError>> {
    Box::pin(async move {
      let stream = invocation.eject_stream();
      let packets = stream.collect::<Vec<_>>().await;
      let mut packets = packets
        .into_iter()
        .collect::<Result<Vec<_>, wick_packet::Error>>()
        .map_err(flow_component::ComponentError::new)?;
      let names = packets.iter().map(|p| p.port().to_owned()).collect::<HashSet<_>>();
      let which_done = packets
        .iter()
        .filter_map(|p| if p.is_done() { Some(p.port().to_owned()) } else { None })
        .collect::<HashSet<_>>();

      for name in names {
        if !which_done.contains(&name) {
          packets.push(Packet::done(name));
        }
      }

      Ok(PacketStream::from(packets))
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}

#[test_logger::test(tokio::test)]
async fn test_done_tolerance() -> Result<()> {
  let config = vec![TestConfigurationBuilder::default()
    .cases(vec![
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![TestPacket::success("a1", None), TestPacket::success("a2", None)])
        .outputs(vec![TestPacket::success("a1", None), TestPacket::success("a2", None)])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![TestPacket::success("b1", None), TestPacket::success("b2", None)])
        .outputs(vec![
          TestPacket::success("b1", None),
          TestPacket::success("b2", None),
          TestPacket::done("b1"),
          TestPacket::done("b2"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![TestPacket::success("c1", None), TestPacket::success("c2", None)])
        .outputs(vec![
          TestPacket::success("c1", None),
          TestPacket::success("c2", None),
          TestPacket::done("c2"),
          TestPacket::done("c1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          TestPacket::success("d1", None),
          TestPacket::done("d1"),
          TestPacket::success("d2", None),
        ])
        .outputs(vec![
          TestPacket::success("d1", None),
          TestPacket::success("d2", None),
          TestPacket::done("d1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          TestPacket::success("e1", None),
          TestPacket::done("e1"),
          TestPacket::success("e2", None),
        ])
        .outputs(vec![
          TestPacket::success("e1", None),
          TestPacket::success("e2", None),
          TestPacket::done("e1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          TestPacket::success("f1", None),
          TestPacket::done("f1"),
          TestPacket::success("f2", None),
        ])
        .outputs(vec![TestPacket::success("f1", None), TestPacket::success("f2", None)])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          TestPacket::success("e1", Some(json!({ "a": 1 }).into())),
          TestPacket::success("e1", Some(json!({ "a": 2 }).into())),
          TestPacket::success("e1", Some(json!({ "a": 3 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 1 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 2 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 3 }).into())),
        ])
        .outputs(vec![
          TestPacket::success("e1", Some(json!({ "a": 1 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 1 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 2 }).into())),
          TestPacket::success("e2", Some(json!({ "b": 3 }).into())),
          TestPacket::success("e1", Some(json!({ "a": 2 }).into())),
          TestPacket::success("e1", Some(json!({ "a": 3 }).into())),
        ])
        .build()?,
    ])
    .build()?];
  let mut suite = TestSuite::from_configuration(&config)?;

  let factory: ComponentFactory = Box::new(move |_config| {
    let task = async move {
      let component = TestComponent::new();
      let component: SharedComponent = Arc::new(component);
      Ok(component)
    };
    Box::pin(task)
  });
  let runners = suite.run(factory, Default::default()).await?;
  for runner in runners {
    runner.print();
    if runner.num_failed() > 0 {
      panic!("Test failed");
    }
  }
  Ok(())
}
