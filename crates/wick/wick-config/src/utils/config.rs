use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use wick_packet::RuntimeConfig;

use crate::config::template_config::Renderable;
use crate::config::{ImportBinding, ImportDefinition, OwnedConfigurationItem, ResourceBinding};
use crate::{Error, Resolver, Result};

pub(crate) type RwOption<T> = Arc<RwLock<Option<T>>>;

pub(crate) fn make_resolver(
  imports: Vec<ImportBinding>,
  resources: Vec<ResourceBinding>,
  runtime_config: Option<RuntimeConfig>,
  env: Option<HashMap<String, String>>,
) -> Box<Resolver> {
  Box::new(move |name| resolve(name, &imports, &resources, runtime_config.as_ref(), env.as_ref()))
}

pub(crate) fn resolve(
  name: &str,
  imports: &[ImportBinding],
  resources: &[ResourceBinding],
  runtime_config: Option<&RuntimeConfig>,
  env: Option<&HashMap<String, String>>,
) -> Result<OwnedConfigurationItem> {
  tracing::trace!("resolving {}, imports: {:?}, resources: {:?}", name, imports, resources);
  if let Some(import) = imports.iter().find(|i| i.id == name) {
    match &import.kind {
      ImportDefinition::Component(component) => {
        let mut component = component.clone();
        return match component.render_config(runtime_config, env) {
          Ok(_) => Ok(OwnedConfigurationItem::Component(component)),
          Err(e) => Err(e),
        };
      }
      ImportDefinition::Types(_) => todo!(),
    }
  }
  if let Some(resource) = resources.iter().find(|i| i.id == name) {
    let mut resource = resource.kind.clone();
    return match resource.render_config(runtime_config, env) {
      Ok(_) => Ok(OwnedConfigurationItem::Resource(resource)),
      Err(e) => Err(e),
    };
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
