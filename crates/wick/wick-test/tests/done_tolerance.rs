use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use flow_component::{Component, RuntimeCallback, SharedComponent};
use serde_json::json;
use tokio_stream::StreamExt;
use wick_config::config::test_case::{PacketData, TestCaseBuilder, TestPacketData};
use wick_config::config::TestConfigurationBuilder;
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
    invocation: Invocation,
    _data: Option<RuntimeConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> flow_component::BoxFuture<std::result::Result<PacketStream, flow_component::ComponentError>> {
    Box::pin(async move {
      let stream = invocation.into_stream();
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
        .inputs(vec![PacketData::success("a1", None), PacketData::success("a2", None)])
        .outputs(vec![
          TestPacketData::success("a1", None),
          TestPacketData::success("a2", None),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![PacketData::success("b1", None), PacketData::success("b2", None)])
        .outputs(vec![
          TestPacketData::success("b1", None),
          TestPacketData::success("b2", None),
          TestPacketData::done("b1"),
          TestPacketData::done("b2"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![PacketData::success("c1", None), PacketData::success("c2", None)])
        .outputs(vec![
          TestPacketData::success("c1", None),
          TestPacketData::success("c2", None),
          TestPacketData::done("c2"),
          TestPacketData::done("c1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          PacketData::success("d1", None),
          PacketData::done("d1"),
          PacketData::success("d2", None),
        ])
        .outputs(vec![
          TestPacketData::success("d1", None),
          TestPacketData::success("d2", None),
          TestPacketData::done("d1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          PacketData::success("e1", None),
          PacketData::done("e1"),
          PacketData::success("e2", None),
        ])
        .outputs(vec![
          TestPacketData::success("e1", None),
          TestPacketData::success("e2", None),
          TestPacketData::done("e1"),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          PacketData::success("f1", None),
          PacketData::done("f1"),
          PacketData::success("f2", None),
        ])
        .outputs(vec![
          TestPacketData::success("f1", None),
          TestPacketData::success("f2", None),
        ])
        .build()?,
      TestCaseBuilder::default()
        .operation("echo")
        .inputs(vec![
          PacketData::success("e1", Some(json!({ "a": 1 }).into())),
          PacketData::success("e1", Some(json!({ "a": 2 }).into())),
          PacketData::success("e1", Some(json!({ "a": 3 }).into())),
          PacketData::success("e2", Some(json!({ "b": 1 }).into())),
          PacketData::success("e2", Some(json!({ "b": 2 }).into())),
          PacketData::success("e2", Some(json!({ "b": 3 }).into())),
        ])
        .outputs(vec![
          TestPacketData::success("e1", Some(json!({ "a": 1 }).into())),
          TestPacketData::success("e2", Some(json!({ "b": 1 }).into())),
          TestPacketData::success("e2", Some(json!({ "b": 2 }).into())),
          TestPacketData::success("e2", Some(json!({ "b": 3 }).into())),
          TestPacketData::success("e1", Some(json!({ "a": 2 }).into())),
          TestPacketData::success("e1", Some(json!({ "a": 3 }).into())),
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
