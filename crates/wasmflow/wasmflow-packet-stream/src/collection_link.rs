use serde::{Deserialize, Serialize};
use wasmflow_entity::Entity;
// use wasmflow_packet::PacketMap;

// type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;

// use wasmflow_output::ComponentOutput;

/// An implementation that encapsulates a collection link that components use to call out to components on other Wasmflow collections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[must_use]
pub struct CollectionLink(Entity, Entity);

impl CollectionLink {
  /// Constructor for a [CollectionLink]
  pub fn new(from: Entity, to: Entity) -> Self {
    Self(from, to)
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> String {
    self.0.url()
  }

  // /// Make a call to the linked collection.
  // pub fn call(
  //   &self,
  //   component: &str,
  //   payload: impl Into<PacketMap>,
  // ) -> BoxedFuture<Result<ComponentOutput, crate::error::Error>> {
  //   let payload = payload.into();
  //   let origin = self.get_origin_url();
  //   let target = Entity::operation(self.1.namespace(), component).url();
  //   Box::pin(async move {
  //     let stream = link_call(&origin, &target, &payload).await?;

  //     Ok(stream)
  //   })
  // }
}

impl std::fmt::Display for CollectionLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.0, self.1)
  }
}

// #[cfg(target_arch = "wasm32")]
// async fn link_call(origin: &str, target: &str, payload: &PacketMap) -> Result<ComponentOutput, crate::error::Error> {
//   let bytes = wasmflow_codec::messagepack::serialize(payload)?;
//   let result = wasmflow_component::guest::wasm::runtime::async_host_call("1", &origin, &target, &bytes)
//     .await
//     .map_err(crate::error::Error::Protocol)?;
//   let packets: Vec<wasmflow_packet::PacketWrapper> = wasmflow_codec::messagepack::deserialize(&result)?;
//   Ok(wasmflow_output::ComponentOutput::new(tokio_stream::iter(packets)))
// }

// #[cfg(not(target_arch = "wasm32"))]
// async fn link_call(_origin: &str, _target: &str, _payload: &PacketMap) -> Result<ComponentOutput, crate::error::Error> {
//   unimplemented!("Link calls from native collections is not implemented yet")
// }
