use actix::{Recipient, SyncArbiter};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::Path};

use crate::native_component_actor::{self, NativeComponentActor};
use crate::vino_component::BoxedComponent;
use crate::{wapc_component_actor, Result};

use crate::{
    oci::fetch_oci_bytes,
    vino_component::{NativeComponent, VinoComponent, WapcComponent},
    RuntimeManifest,
};

use crate::{dispatch::Invocation, wapc_component_actor::WapcComponentActor};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchematicDefinition {
    pub name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external: Vec<ExternalComponentDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<ConnectionDefinition>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub constraints: HashMap<String, String>,
}

impl SchematicDefinition {
    pub fn new(
        name: &str,
        external: Vec<ExternalComponentDefinition>,
        components: Vec<(String, ComponentDefinition)>,
        connections: Vec<ConnectionDefinition>,
        constraints: Vec<(String, String)>,
    ) -> Self {
        SchematicDefinition {
            name: name.to_string(),
            external,
            components: components.iter().cloned().collect(),
            connections,
            constraints: constraints.iter().cloned().collect(),
        }
    }
    pub fn get_output_names(&self) -> Vec<String> {
        self.connections
            .iter()
            .filter(|conn| conn.to.instance == crate::SCHEMATIC_OUTPUT)
            .map(|conn| conn.to.port.to_string())
            .collect()
    }
    pub fn id_to_ref(&self, id: &str) -> Result<String> {
        if id.starts_with(crate::VINO_NAMESPACE) {
            Ok(id.to_string())
        } else {
            for component in &self.external {
                if id == component.key || Some(id.to_string()) == component.alias {
                    return Ok(component.component_ref.to_string());
                }
            }
            return Err(anyhow!("No external component found with alias or key {}", id).into());
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalComponentDefinition {
    pub alias: Option<String>,
    #[serde(rename = "ref")]
    pub component_ref: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    pub metadata: Option<String>,
    #[serde(rename = "id")]
    pub id: String,
}

impl ComponentDefinition {
    pub fn new(actor_ref: impl ToString, metadata: Option<String>) -> Self {
        ComponentDefinition {
            id: actor_ref.to_string(),
            metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDefinition {
    pub from: ConnectionTargetDefinition,
    pub to: ConnectionTargetDefinition,
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

impl ConnectionDefinition {
    pub fn new(
        from: impl Into<ConnectionTargetDefinition>,
        to: impl Into<ConnectionTargetDefinition>,
    ) -> Self {
        ConnectionDefinition {
            from: from.into(),
            to: to.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTargetDefinition {
    pub instance: String,
    pub port: String,
}

impl ConnectionTargetDefinition {
    pub fn new(instance: String, port: String) -> Self {
        ConnectionTargetDefinition { instance, port }
    }
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

pub(crate) async fn get_components(
    manifest: &RuntimeManifest,
    seed: String,
    allow_latest: bool,
    allowed_insecure: &[String],
) -> Result<Vec<(BoxedComponent, Recipient<Invocation>)>> {
    let mut v: Vec<(BoxedComponent, Recipient<Invocation>)> = Vec::new();
    for schematic in &manifest.schematics {
        for comp in schematic.components.values() {
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
            Err(e) => Err(anyhow!("Could not read file {}:{}", comp_ref, e.to_string()).into()),
        }
    } else if comp_ref.starts_with("vino::") {
        match NativeComponent::from_id(comp_ref.to_string()) {
            Ok(component) => {
                trace!("Starting native component '{}'", component.name(),);
                let actor = SyncArbiter::start(1, NativeComponentActor::default);
                actor
                    .send(native_component_actor::Initialize {
                        name: component.name(),
                    })
                    .await??;
                let recipient = actor.recipient::<Invocation>();

                Ok((Box::new(component), recipient))
            }
            Err(e) => Err(anyhow!("Could not load native component {}: {}", comp_ref, e).into()),
        }
    } else {
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
            Err(_) => Err(anyhow!(
                "Could not find {} component on disk or in registry",
                comp_ref,
            )
            .into()),
        }
    };
    component
}
