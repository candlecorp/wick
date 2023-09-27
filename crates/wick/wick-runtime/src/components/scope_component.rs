use uuid::Uuid;
use wick_packet::Entity;

use crate::dev::prelude::*;

#[derive(Debug, Default)]
struct State {}

#[derive(Clone, Debug)]
pub struct ScopeComponent {
  scope_id: Uuid,
  signature: ComponentSignature,
}

impl ScopeComponent {
  #[must_use]
  pub fn new(scope_id: Uuid) -> Self {
    let addr = Scope::for_id(&scope_id).unwrap();

    let signature = addr.get_signature().unwrap();

    Self { scope_id, signature }
  }
}

impl Component for ScopeComponent {
  fn handle(
    &self,
    mut invocation: Invocation,
    config: Option<RuntimeConfig>,
    _callback: LocalScope,
  ) -> flow_component::BoxFuture<Result<PacketStream, flow_component::ComponentError>> {
    let target_url = invocation.target().url();

    invocation.trace(|| {
      debug!(
        scope_id = %self.scope_id,
        target =  %invocation.target(),
        "scope:invoke",
      );
    });

    Box::pin(async move {
      let scope = Scope::for_id(&self.scope_id)
        .ok_or_else(|| flow_component::ComponentError::msg(format!("scope '{}' not found", target_url)))?;

      let target_component = invocation.target().component_id().to_owned();
      if target_component != scope.namespace() {
        debug!(
          orig_target = target_component,
          runtime = scope.namespace(),
          "translating invocation target to scope namespace"
        );
        let new_target = Entity::operation(scope.namespace(), invocation.target().operation_id());
        invocation = invocation.redirect(new_target);
      }

      invocation.trace(|| trace!(target = %target_url, "invoking"));

      let result: InvocationResponse = scope
        .invoke(invocation, config)
        .map_err(flow_component::ComponentError::new)?
        .await
        .map_err(flow_component::ComponentError::new)?;

      match result.ok() {
        Ok(stream) => Ok(stream),
        Err(msg) => Err(flow_component::ComponentError::new(msg)),
      }
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}

#[cfg(test)]
mod tests {

  use futures::StreamExt;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  type Result<T> = anyhow::Result<T>;

  async fn request_log(component: &ScopeComponent, data: &str) -> Result<String> {
    let stream = packet_stream!(("MAIN_IN", data));

    let invocation = Invocation::test(file!(), Entity::local("simple"), stream, None)?;
    let outputs = component
      .handle(invocation, Default::default(), Default::default())
      .await?;
    let mut packets: Vec<_> = outputs.collect().await;
    println!("packets: {:#?}", packets);
    let _ = packets.pop();
    let actual = packets.pop().unwrap().unwrap();

    println!("doc_id: {:?}", actual);
    assert_eq!(actual, Packet::encode("MAIN_OUT", data));
    Ok(actual.payload.decode().unwrap())
  }

  #[test_logger::test(tokio::test)]
  async fn test_request_log() -> Result<()> {
    let (_, scope_id) = init_scope_from_yaml("./manifests/v0/simple.yaml").await?;

    let component = ScopeComponent::new(scope_id);
    let user_data = "string to log";
    let result = request_log(&component, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> Result<()> {
    let (_, scope_id) = init_scope_from_yaml("./manifests/v0/simple.yaml").await?;
    let component = ScopeComponent::new(scope_id);
    let sig = component.signature();
    println!("operations in scope : {:?}", sig);
    assert_eq!(sig.operations.len(), 1);
    Ok(())
  }
}
