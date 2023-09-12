use flow_component::{Component, ComponentError, RuntimeCallback};
use futures::FutureExt;
use tokio::spawn;
use tokio_stream::StreamExt;
use wick_interface_types::{operation, ComponentSignature};
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::graph::types::Node;
use crate::graph::NodeDecorator;
use crate::BoxFuture;

#[derive(Debug)]
pub(crate) struct NullComponent {
  signature: ComponentSignature,
}

impl NodeDecorator for NullComponent {
  fn decorate(node: &mut Node) -> Result<(), String> {
    node.add_input("input");
    Ok(())
  }
}

impl NullComponent {
  pub(crate) const ID: &str = "__null__";

  pub(crate) fn new() -> Self {
    let mut this = Self {
      signature: ComponentSignature::new_named(Self::ID).set_version("0.0.0"),
    };
    this.signature = this
      .signature
      .add_operation(operation! {"drop"=>{inputs:{"input"=>"object"},outputs:{},}});

    this
  }
}

impl Component for NullComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _data: Option<RuntimeConfig>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    spawn(async move {
      let (invocation, mut stream) = invocation.split();
      while let Some(p) = stream.next().await {
        match p {
          Err(e) => invocation.trace(|| error!("received error on dropped stream: {}", e)),
          Ok(p) if p.is_error() => {
            invocation.trace(|| debug!("received error packet on dropped stream: {:?}", p.unwrap_err()));
          }
          _ => {}
        }
      }
    });
    async move { Ok(PacketStream::empty()) }.boxed()
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}
