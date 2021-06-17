// This file is generated, do not edit
use vino_provider::VinoProviderComponent;
pub(crate) mod generated;

pub mod test;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "vino::test::provider" => Some(Box::new(test::Component::default())),
    _ => None,
  }
}
