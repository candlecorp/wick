use std::collections::{
  HashMap,
  HashSet,
};
use std::convert::{
  TryFrom,
  TryInto,
};
use std::sync::Arc;

use parking_lot::RwLock;
use vino_provider::native::prelude::*;

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, NetworkModelError>;

type Namespace = String;

#[derive(Debug, Clone)]
pub(crate) struct NetworkModel {
  definition: NetworkDefinition,
  schematics: Vec<Arc<RwLock<SchematicModel>>>,
  providers: HashMap<Namespace, Option<ProviderModel>>,
  state: Option<LoadedState>,
}

#[derive(Debug, Clone)]
struct LoadedState {
  provider_signatures: Vec<ProviderSignature>,
}

impl NetworkModel {
  pub(crate) fn get_name(&self) -> Option<&String> {
    self.definition.name.as_ref()
  }

  pub(crate) fn get_schematics(&self) -> &Vec<Arc<RwLock<SchematicModel>>> {
    &self.schematics
  }

  pub(crate) fn get_schematic(&self, name: &str) -> Option<&Arc<RwLock<SchematicModel>>> {
    self
      .schematics
      .iter()
      .find(|sc| sc.read().get_name() == name)
  }

  pub(crate) fn update_providers(
    &mut self,
    providers: HashMap<String, Option<ProviderModel>>,
  ) -> Result<()> {
    trace!(
      "MODEL:NETWORK:PROVIDERS:[{}]",
      providers.iter().map(|(ns, _)| ns).join(", "),
    );

    for schematic in self.schematics.iter_mut() {
      schematic.write().update_providers(providers.clone());
    }
    self.providers = providers;
    // ensure state is reset;
    self.state = None;
    Ok(())
  }

  pub(crate) fn update_self_component(
    &mut self,
    name: String,
    signature: ComponentSignature,
  ) -> Result<()> {
    let provider = self
      .providers
      .entry(name.clone())
      .or_insert_with(|| Some(ProviderModel::default()));
    let model = provider.get_or_insert(ProviderModel::default());

    model.components.insert(
      name.clone(),
      ComponentSignature {
        name,
        inputs: signature.inputs,
        outputs: signature.outputs,
      }
      .into(),
    );

    for schematic in self.schematics.iter_mut() {
      schematic.write().commit_self_provider(model.clone())?;
    }

    // ensure state is reset;
    self.state = None;
    Ok(())
  }

  pub(crate) fn finalize(&mut self) -> Result<()> {
    self.populate_state()
  }

  fn populate_state(&mut self) -> Result<()> {
    let provider_signatures = self
      .providers
      .iter()
      .filter_map(|(ns, provider_model)| {
        provider_model
          .as_ref()
          .map(|model| model.get_signature(Some(ns.clone())))
      })
      .collect();
    for schematic in &self.schematics {
      schematic.write().finalize()?;
    }
    self.state = Some(LoadedState {
      provider_signatures,
    });
    Ok(())
  }

  #[allow(unused)]
  pub(crate) fn get_resolution_order(&self) -> Result<Vec<Vec<String>>> {
    let mut order = vec![];
    let mut will_resolve = HashSet::new();
    let mut schematics = self.get_schematics().clone();

    let mut cycle = 0;
    let mut num_unresolved = schematics.len();
    while cycle < 5 {
      let mut unresolved = vec![];
      let mut next_batch = vec![];
      for schematic in schematics {
        let sc = schematic.read();
        let mut resolvable = true;
        for component in sc.get_component_definitions() {
          let is_self_referential = component.namespace == SELF_NAMESPACE;
          let reference_will_have_resolved = will_resolve.contains(&component.name);
          if is_self_referential && !reference_will_have_resolved {
            resolvable = false;
          }
        }
        if resolvable {
          will_resolve.insert(sc.get_name());
          next_batch.push(sc.get_name());
        } else {
          unresolved.push(schematic.clone());
        }
      }
      if !next_batch.is_empty() {
        order.push(next_batch);
        next_batch = vec![];
      }
      schematics = unresolved;
      if schematics.is_empty() {
        break;
      }
      if num_unresolved == schematics.len() {
        cycle += 1;
      } else {
        num_unresolved = schematics.len();
      }
    }

    Ok(order)
  }
}

impl TryFrom<NetworkDefinition> for NetworkModel {
  type Error = NetworkModelError;

  fn try_from(definition: NetworkDefinition) -> Result<Self> {
    let mut schematics = Vec::new();

    for schematic in &definition.schematics {
      schematics.push(Arc::new(RwLock::new(
        schematic
          .clone()
          .try_into()
          .map_err(NetworkModelError::SchematicModelError)?,
      )));
    }

    Ok(Self {
      definition,
      providers: HashMap::new(),
      schematics,
      state: None,
    })
  }
}

#[cfg(test)]
mod tests {

  #[allow(unused_imports)]
  use crate::test::prelude::{
    assert_eq,
    *,
  };

  #[test_logger::test]
  fn test_basics() -> TestResult<()> {
    let network = load_network_definition("./src/models/test-manifests/logger.yaml")?;
    let _ = NetworkModel::try_from(network)?;

    Ok(())
  }

  #[test_logger::test]
  fn test_resolution_order() -> TestResult<()> {
    let network = load_network_definition("./manifests/v0/nested-schematics.yaml")?;
    let model = NetworkModel::try_from(network)?;
    let resolution_order = model.get_resolution_order()?;
    assert_eq!(resolution_order, vec![vec!["child"], vec!["nested_parent"]]);

    Ok(())
  }

  #[test_logger::test]
  fn test_find_defaults() -> TestResult<()> {
    Ok(())
  }
}
