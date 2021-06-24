// This file is generated, do not edit
use vino_provider::{
  Component,
  VinoProviderComponent,
};
pub(crate) mod generated;

pub mod add;
pub mod bcrypt;
pub mod log;
pub mod string_to_bytes;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "log" => Some(Box::new(log::Component::default())),
    "add" => Some(Box::new(add::Component::default())),
    "string-to-bytes" => Some(Box::new(string_to_bytes::Component::default())),
    "bcrypt" => Some(Box::new(bcrypt::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<Component> {
  vec![
    Component {
      name: "log".to_string(),
      inputs: log::inputs_list().into_iter().map(From::from).collect(),
      outputs: log::outputs_list().into_iter().map(From::from).collect(),
    },
    Component {
      name: "add".to_string(),
      inputs: add::inputs_list().into_iter().map(From::from).collect(),
      outputs: add::outputs_list().into_iter().map(From::from).collect(),
    },
    Component {
      name: "string-to-bytes".to_string(),
      inputs: string_to_bytes::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: string_to_bytes::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "bcrypt".to_string(),
      inputs: bcrypt::inputs_list().into_iter().map(From::from).collect(),
      outputs: bcrypt::outputs_list().into_iter().map(From::from).collect(),
    },
  ]
}
