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
    "add" => Some(Box::new(generated::add::Component::default())),
    "bcrypt" => Some(Box::new(generated::bcrypt::Component::default())),
    "log" => Some(Box::new(generated::log::Component::default())),
    "string-to-bytes" => Some(Box::new(generated::string_to_bytes::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<Component> {
  vec![
    Component {
      name: "add".to_string(),
      inputs: generated::add::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::add::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "bcrypt".to_string(),
      inputs: generated::bcrypt::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::bcrypt::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "log".to_string(),
      inputs: generated::log::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::log::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "string-to-bytes".to_string(),
      inputs: generated::string_to_bytes::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::string_to_bytes::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}
