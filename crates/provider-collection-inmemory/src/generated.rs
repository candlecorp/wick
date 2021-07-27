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
    "add-item" => Some(Box::new(self::add_item::Component::default())),
    "get-item" => Some(Box::new(self::get_item::Component::default())),
    "list-items" => Some(Box::new(self::list_items::Component::default())),
    "rm-item" => Some(Box::new(self::rm_item::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    ComponentSignature {
      name: "add-item".to_owned(),
      inputs: vino_interfaces_collection::add_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_collection::add_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "get-item".to_owned(),
      inputs: vino_interfaces_collection::get_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_collection::get_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "list-items".to_owned(),
      inputs: vino_interfaces_collection::list_items::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_collection::list_items::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "rm-item".to_owned(),
      inputs: vino_interfaces_collection::rm_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: vino_interfaces_collection::rm_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}

pub(crate) mod add_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_collection::add_item::*;
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
      format!("vino::{}", "add-item")
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
      let result = crate::components::add_item::job(inputs, outputs, context).await;
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
pub(crate) mod get_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_collection::get_item::*;
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
      format!("vino::{}", "get-item")
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
      let result = crate::components::get_item::job(inputs, outputs, context).await;
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
pub(crate) mod list_items {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_collection::list_items::*;
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
      format!("vino::{}", "list-items")
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
      let result = crate::components::list_items::job(inputs, outputs, context).await;
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
pub(crate) mod rm_item {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interfaces_collection::rm_item::*;
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
      format!("vino::{}", "rm-item")
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
      let result = crate::components::rm_item::job(inputs, outputs, context).await;
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
