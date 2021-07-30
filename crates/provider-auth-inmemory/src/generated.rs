/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub(crate) use vino_provider::native::prelude::*;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn NativeComponent<State = crate::State> + Sync + Send>> {
  match name {
    "authenticate" => Some(Box::new(self::authenticate::Component::default())),
    "create-user" => Some(Box::new(self::create_user::Component::default())),
    "get-id" => Some(Box::new(self::get_id::Component::default())),
    "has-permission" => Some(Box::new(self::has_permission::Component::default())),
    "list-permissions" => Some(Box::new(self::list_permissions::Component::default())),
    "list-users" => Some(Box::new(self::list_users::Component::default())),
    "remove-user" => Some(Box::new(self::remove_user::Component::default())),
    "update-permissions" => Some(Box::new(self::update_permissions::Component::default())),
    "validate-session" => Some(Box::new(self::validate_session::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    ComponentSignature {
      name: "authenticate".to_owned(),
      inputs: vino_interfaces_authentication::authenticate::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::authenticate::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "create-user".to_owned(),
      inputs: vino_interfaces_authentication::create_user::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::create_user::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "get-id".to_owned(),
      inputs: vino_interfaces_authentication::get_id::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::get_id::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "has-permission".to_owned(),
      inputs: vino_interfaces_authentication::has_permission::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::has_permission::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "list-permissions".to_owned(),
      inputs: vino_interfaces_authentication::list_permissions::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::list_permissions::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "list-users".to_owned(),
      inputs: vino_interfaces_authentication::list_users::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::list_users::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "remove-user".to_owned(),
      inputs: vino_interfaces_authentication::remove_user::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::remove_user::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "update-permissions".to_owned(),
      inputs: vino_interfaces_authentication::update_permissions::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::update_permissions::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "validate-session".to_owned(),
      inputs: vino_interfaces_authentication::validate_session::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_authentication::validate_session::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}

pub(crate) mod authenticate {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::authenticate::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::create_user::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::get_id::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::has_permission::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::list_permissions::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::list_users::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::remove_user::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::update_permissions::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
  use vino_interfaces_authentication::validate_session::*;
  use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
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
