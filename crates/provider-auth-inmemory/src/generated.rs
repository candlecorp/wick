/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::{
  ComponentSignature,
  VinoProviderComponent,
};

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::authenticate::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "authenticate")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::authenticate::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod create_user {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::create_user::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "create-user")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::create_user::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod get_id {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::get_id::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "get-id")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::get_id::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod has_permission {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::has_permission::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "has-permission")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::has_permission::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod list_permissions {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::list_permissions::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "list-permissions")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::list_permissions::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod list_users {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::list_users::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "list-users")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::list_users::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod remove_user {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::remove_user::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "remove-user")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::remove_user::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod update_permissions {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::update_permissions::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "update-permissions")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::update_permissions::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod validate_session {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_authentication::validate_session::*;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  use vino_rpc::port::PortStream;

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "validate-session")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::validate_session::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
