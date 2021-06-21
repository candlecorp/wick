use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use actix::{
  Recipient,
  SyncArbiter,
};
use serde::{
  Deserialize,
  Serialize,
};
use vino_manifest::SchematicManifest;

use crate::components::native_component_actor::{
  self,
  NativeComponentActor,
};
use crate::components::vino_component::{
  BoxedComponent,
  NativeComponent,
  VinoComponent,
  WapcComponent,
};
use crate::components::wapc_component_actor;
use crate::components::wapc_component_actor::WapcComponentActor;
use crate::dispatch::Invocation;
use crate::network::ComponentMetadata;
use crate::network_definition::NetworkDefinition;
use crate::util::oci::fetch_oci_bytes;
use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, Default)]
pub struct SchematicDefinition {
  pub name: String,
  pub(crate) external: Vec<ExternalComponentDefinition>,
  pub(crate) components: HashMap<String, ComponentDefinition>,
  pub(crate) connections: Vec<ConnectionDefinition>,
  pub(crate) constraints: HashMap<String, String>,
}

impl SchematicDefinition {
  pub(crate) fn new(manifest: &SchematicManifest) -> Self {
    match manifest {
      SchematicManifest::V0(manifest) => Self {
        name: manifest.name.clone(),
        components: manifest
          .components
          .clone()
          .into_iter()
          .map(|(key, val)| (key, val.into()))
          .collect(),
        connections: manifest
          .connections
          .clone()
          .into_iter()
          .map(|def| def.into())
          .collect(),
        constraints: manifest.constraints.clone().into_iter().collect(),
        external: manifest
          .external
          .clone()
          .into_iter()
          .map(|def| def.into())
          .collect(),
      },
    }
  }
  pub(crate) fn get_name(&self) -> String {
    self.name.clone()
  }
  pub(crate) fn get_component(&self, name: &str) -> Option<&ComponentDefinition> {
    self.components.get(name)
  }

  pub(crate) fn get_output_names(&self) -> Vec<String> {
    self
      .connections
      .iter()
      .filter(|conn| conn.to.instance == crate::SCHEMATIC_OUTPUT)
      .map(|conn| conn.to.port.to_string())
      .collect()
  }
  pub(crate) fn id_to_ref(&self, id: &str) -> Result<String> {
    if id.starts_with(crate::VINO_NAMESPACE) {
      Ok(id.to_string())
    } else {
      for component in &self.external {
        if id == component.key || Some(id.to_string()) == component.alias {
          return Ok(component.reference.to_string());
        }
      }
      Err(Error::SchematicError(format!(
        "No external component found with alias or key {}",
        id
      )))
    }
  }
}

impl From<vino_manifest::v0::SchematicManifest> for SchematicDefinition {
  fn from(def: vino_manifest::v0::SchematicManifest) -> Self {
    Self::new(&vino_manifest::SchematicManifest::V0(def))
  }
}

#[derive(Debug, Clone)]
pub struct ExternalComponentDefinition {
  pub alias: Option<String>,
  pub reference: String,
  pub key: String,
}

impl From<vino_manifest::v0::ExternalComponentDefinition> for ExternalComponentDefinition {
  fn from(def: vino_manifest::v0::ExternalComponentDefinition) -> Self {
    Self {
      alias: def.alias,
      key: def.key,
      reference: def.reference,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
  pub metadata: Option<String>,
  pub id: String,
}

impl From<vino_manifest::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: vino_manifest::v0::ComponentDefinition) -> Self {
    ComponentDefinition {
      id: def.id,
      metadata: None,
    }
  }
}

impl From<&vino_manifest::v0::ComponentDefinition> for ComponentDefinition {
  fn from(def: &vino_manifest::v0::ComponentDefinition) -> Self {
    ComponentDefinition {
      id: def.id.to_string(),
      metadata: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConnectionDefinition {
  pub from: ConnectionTargetDefinition,
  pub to: ConnectionTargetDefinition,
}

impl From<vino_manifest::v0::ConnectionDefinition> for ConnectionDefinition {
  fn from(def: vino_manifest::v0::ConnectionDefinition) -> Self {
    ConnectionDefinition {
      from: def.from.into(),
      to: def.to.into(),
    }
  }
}

impl Display for ConnectionDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

impl Display for ConnectionTargetDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.instance, self.port)
  }
}

impl From<vino_manifest::v0::ConnectionTargetDefinition> for ConnectionTargetDefinition {
  fn from(def: vino_manifest::v0::ConnectionTargetDefinition) -> Self {
    ConnectionTargetDefinition {
      instance: def.instance,
      port: def.port,
    }
  }
}

impl ConnectionDefinition {
  pub fn print_all(list: &[Self]) -> String {
    list
      .iter()
      .map(|c| c.to_string())
      .collect::<Vec<String>>()
      .join(", ")
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTargetDefinition {
  pub instance: String,
  pub port: String,
}

impl<T, U> From<(T, U)> for ConnectionTargetDefinition
where
  T: Display,
  U: Display,
{
  fn from((instance, port): (T, U)) -> Self {
    ConnectionTargetDefinition {
      instance: instance.to_string(),
      port: port.to_string(),
    }
  }
}

pub(crate) async fn _get_components(
  network: &NetworkDefinition,
  seed: String,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<Vec<(BoxedComponent, Recipient<Invocation>)>> {
  let mut v: Vec<(BoxedComponent, Recipient<Invocation>)> = Vec::new();
  debug!("getting components {:?}", network);

  for schematic in &network.schematics {
    debug!("{:?}", schematic);
    for comp in schematic.components.values() {
      debug!("{:?}", comp);
      let component_ref = schematic.id_to_ref(&comp.id)?;
      let component = get_component(
        component_ref.to_string(),
        seed.clone(),
        allow_latest,
        allowed_insecure,
      )
      .await?;
      v.push(component);
    }
  }
  Ok(v)
}

pub(crate) async fn get_components_for_schematic(
  schematic: SchematicDefinition,
  seed: String,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<HashMap<String, ComponentMetadata>> {
  let mut metadata_map: HashMap<String, ComponentMetadata> = HashMap::new();
  debug!("getting components for schematic {:?}", schematic);

  for comp in schematic.components.values() {
    debug!("{:?}", comp);
    let component_ref = schematic.id_to_ref(&comp.id)?;
    let (component, addr) = get_component(
      component_ref.to_string(),
      seed.clone(),
      allow_latest,
      &allowed_insecure,
    )
    .await?;
    metadata_map.insert(
      component.id(),
      ComponentMetadata {
        inputs: component.get_inputs(),
        outputs: component.get_outputs(),
        addr,
      },
    );
  }
  Ok(metadata_map)
}

pub(crate) async fn get_component(
  comp_ref: String,
  seed: String,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<(BoxedComponent, Recipient<Invocation>)> {
  let p = Path::new(&comp_ref);
  let component: Result<(BoxedComponent, Recipient<Invocation>)> = if p.exists() {
    // read actor from disk
    match WapcComponent::from_file(p) {
      Ok(component) => {
        trace!(
          "Starting wapc component '{}' from file {}",
          component.name(),
          p.to_string_lossy()
        );
        let actor = SyncArbiter::start(1, WapcComponentActor::default);
        actor
          .send(wapc_component_actor::Initialize {
            actor_bytes: component.bytes.clone(),
            signing_seed: seed,
          })
          .await??;
        let recipient = actor.recipient::<Invocation>();
        Ok((Box::new(component), recipient))
      }
      Err(e) => Err(Error::SchematicError(format!(
        "Could not read file {}:{}",
        comp_ref,
        e.to_string()
      ))),
    }
  } else {
    let providers = vec!["vino".to_string()];
    for namespace in providers {
      if comp_ref.starts_with(&format!("{}::", namespace)) {
        trace!(
          "registering component under the {} provider namespace",
          namespace
        );
        let name = str::replace(&comp_ref, &format!("{}::", namespace), "");
        // todo temporary and very hacky
        if namespace == "vino" {
          match NativeComponent::from_id(namespace, name) {
            Ok(component) => {
              trace!("Starting native component '{}'", component.name(),);
              let actor = SyncArbiter::start(1, NativeComponentActor::default);
              actor
                .send(native_component_actor::Initialize {
                  name: component.name(),
                  signing_seed: seed,
                })
                .await??;
              let recipient = actor.recipient::<Invocation>();

              return Ok((Box::new(component), recipient));
            }
            Err(e) => {
              return Err(Error::SchematicError(format!(
                "Could not load native component {}: {}",
                comp_ref, e
              )))
            }
          }
        }
      }
    }
    // load actor from OCI
    let component = fetch_oci_bytes(&comp_ref, allow_latest, allowed_insecure)
      .await
      .and_then(|bytes| WapcComponent::from_slice(&bytes));
    match component {
      Ok(component) => {
        trace!(
          "Starting wapc component '{}' from URL {}",
          component.name(),
          comp_ref
        );

        let actor = SyncArbiter::start(1, WapcComponentActor::default);
        actor
          .send(wapc_component_actor::Initialize {
            actor_bytes: component.bytes.clone(),
            signing_seed: seed,
          })
          .await??;

        let recipient = actor.recipient::<Invocation>();
        Ok((Box::new(component), recipient))
      }
      Err(_) => Err(Error::SchematicError(format!(
        "Could not find component '{}' on disk or in registry",
        comp_ref,
      ))),
    }
  };
  component
}
