use std::collections::HashMap;

use crate::dev::prelude::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderModel {
  pub(crate) components: HashMap<String, ComponentModel>,
}

impl WithSignature<ProviderSignature> for ProviderModel {
  fn get_signature(&self, name: Option<String>) -> ProviderSignature {
    let map: HashMap<String, ComponentSignature> = self
      .components
      .values()
      .map(|model| (model.name_owned(), model.get_signature()))
      .collect();
    ProviderSignature {
      types: HashMap::new().into(),
      name: name.unwrap_or_default(),
      components: map.into(),
    }
  }
}

impl From<ProviderSignature> for ProviderModel {
  fn from(sig: ProviderSignature) -> Self {
    let map: HashMap<String, ComponentModel> = sig
      .components
      .into_inner()
      .into_iter()
      .map(|(k, v)| (k, v.into()))
      .collect();
    Self { components: map }
  }
}
