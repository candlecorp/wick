/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub(crate) use vino_provider::native::prelude::*;

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    vino_interface_collection::add_item::signature(),
    vino_interface_collection::get_item::signature(),
    vino_interface_collection::list_items::signature(),
    vino_interface_collection::rm_item::signature(),
  ]
}

#[derive(Debug)]
pub(crate) struct Dispatcher {}
#[async_trait]
impl Dispatch for Dispatcher {
  type Context = crate::Context;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>> {
    let result = match op {
      "add-item" => {
        self::add_item::Component::default()
          .execute(context, data)
          .await
      }
      "get-item" => {
        self::get_item::Component::default()
          .execute(context, data)
          .await
      }
      "list-items" => {
        self::list_items::Component::default()
          .execute(context, data)
          .await
      }
      "rm-item" => {
        self::rm_item::Component::default()
          .execute(context, data)
          .await
      }
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub(crate) mod add_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_collection::add_item::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::add_item::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod get_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_collection::get_item::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::get_item::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod list_items {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_collection::list_items::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::list_items::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod rm_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_collection::rm_item::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::rm_item::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
