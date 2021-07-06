// This file is generated, do not edit
use vino_provider::{
  ComponentSignature,
  VinoProviderComponent,
};
pub(crate) mod generated;

pub(crate) mod test_component;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "test-component" => Some(Box::new(generated::test_component::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![ComponentSignature {
    name: "test-component".to_string(),
    inputs: generated::test_component::inputs_list()
      .into_iter()
      .map(From::from)
      .collect(),
    outputs: generated::test_component::outputs_list()
      .into_iter()
      .map(From::from)
      .collect(),
  }]
}
