// This file is generated, do not edit
use vino_provider::Component;
pub mod generated;

pub fn get_all_components() -> Vec<Component> {
  vec![
    Component {
      name: "add-item".to_string(),
      inputs: generated::add_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::add_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "get-item".to_string(),
      inputs: generated::get_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::get_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    Component {
      name: "list-items".to_string(),
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
