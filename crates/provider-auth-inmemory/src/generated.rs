/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub(crate) use vino_provider::native::prelude::*;

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    vino_interface_authentication::authenticate::signature(),
    vino_interface_authentication::create_user::signature(),
    vino_interface_authentication::get_id::signature(),
    vino_interface_authentication::has_permission::signature(),
    vino_interface_authentication::list_permissions::signature(),
    vino_interface_authentication::list_users::signature(),
    vino_interface_authentication::remove_user::signature(),
    vino_interface_authentication::update_permissions::signature(),
    vino_interface_authentication::validate_session::signature(),
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
      "authenticate" => {
        self::authenticate::Component::default()
          .execute(context, data)
          .await
      }
      "create-user" => {
        self::create_user::Component::default()
          .execute(context, data)
          .await
      }
      "get-id" => {
        self::get_id::Component::default()
          .execute(context, data)
          .await
      }
      "has-permission" => {
        self::has_permission::Component::default()
          .execute(context, data)
          .await
      }
      "list-permissions" => {
        self::list_permissions::Component::default()
          .execute(context, data)
          .await
      }
      "list-users" => {
        self::list_users::Component::default()
          .execute(context, data)
          .await
      }
      "remove-user" => {
        self::remove_user::Component::default()
          .execute(context, data)
          .await
      }
      "update-permissions" => {
        self::update_permissions::Component::default()
          .execute(context, data)
          .await
      }
      "validate-session" => {
        self::validate_session::Component::default()
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

pub(crate) mod authenticate {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::authenticate::*;
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
      let result = crate::components::authenticate::job(inputs, outputs, context).await;
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
pub(crate) mod create_user {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::create_user::*;
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
      let result = crate::components::create_user::job(inputs, outputs, context).await;
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
pub(crate) mod get_id {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::get_id::*;
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
      let result = crate::components::get_id::job(inputs, outputs, context).await;
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
pub(crate) mod has_permission {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::has_permission::*;
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
      let result = crate::components::has_permission::job(inputs, outputs, context).await;
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
pub(crate) mod list_permissions {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::list_permissions::*;
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
      let result = crate::components::list_permissions::job(inputs, outputs, context).await;
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
pub(crate) mod list_users {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::list_users::*;
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
      let result = crate::components::list_users::job(inputs, outputs, context).await;
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
pub(crate) mod remove_user {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::remove_user::*;
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
      let result = crate::components::remove_user::job(inputs, outputs, context).await;
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
pub(crate) mod update_permissions {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::update_permissions::*;
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
      let result = crate::components::update_permissions::job(inputs, outputs, context).await;
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
pub(crate) mod validate_session {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_authentication::validate_session::*;
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
      let result = crate::components::validate_session::job(inputs, outputs, context).await;
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
