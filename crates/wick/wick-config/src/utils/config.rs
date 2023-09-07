use std::sync::Arc;

use parking_lot::RwLock;

use crate::config::{Binding, ImportDefinition, OwnedConfigurationItem, ResourceDefinition};
use crate::{Error, Resolver, Result};

pub(crate) type RwOption<T> = Arc<RwLock<Option<T>>>;

pub(crate) fn make_resolver(
  imports: Vec<Binding<ImportDefinition>>,
  resources: Vec<Binding<ResourceDefinition>>,
) -> Box<Resolver> {
  Box::new(move |name| resolve(name, &imports, &resources))
}

pub(crate) fn resolve(
  name: &str,
  imports: &[Binding<ImportDefinition>],
  resources: &[Binding<ResourceDefinition>],
) -> Result<OwnedConfigurationItem> {
  tracing::trace!("resolving {}, imports: {:?}, resources: {:?}", name, imports, resources);
  if let Some(import) = imports.iter().find(|i| i.id == name) {
    match &import.kind {
      ImportDefinition::Component(component) => {
        let component = component.clone();

        return Ok(OwnedConfigurationItem::Component(component));
      }
      ImportDefinition::Types(_) => todo!(),
    }
  }
  if let Some(resource) = resources.iter().find(|i| i.id == name) {
    let resource = resource.kind.clone();
    return Ok(OwnedConfigurationItem::Resource(resource));
  }
  Err(Error::IdNotFound {
    id: name.to_owned(),
    ids: [
      imports.iter().map(|i| i.id().to_owned()).collect::<Vec<_>>(),
      resources.iter().map(|i| i.id().to_owned()).collect::<Vec<_>>(),
    ]
    .concat(),
  })
}
