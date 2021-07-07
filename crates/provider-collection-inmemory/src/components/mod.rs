// This file is generated, do not edit
use vino_provider::{
  ComponentSignature,
  VinoProviderComponent,
};
pub(crate) mod generated;

pub(crate) mod add_item;
pub(crate) mod get_item;
pub(crate) mod list_items;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "add-item" => Some(Box::new(generated::add_item::Component::default())),
    "get-item" => Some(Box::new(generated::get_item::Component::default())),
    "list-items" => Some(Box::new(generated::list_items::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    ComponentSignature {
      name: "add-item".to_owned(),
      inputs: generated::add_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::add_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "get-item".to_owned(),
      inputs: generated::get_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::get_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "list-items".to_owned(),
      inputs: generated::list_items::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::list_items::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}
