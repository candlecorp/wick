use crate::Result;
use anyhow::Context;
use std::collections::HashMap;
use wasmcloud_host::vino::{ComponentDefinition, ConnectionDefinition, SchematicDefinition};
use wasmcloud_host::{Actor, HostManifest};

pub fn generate_run_manifest(
    actor_ref: String,
    actor: Actor,
    input: &HashMap<String, serde_json::Value>,
) -> Result<HostManifest> {
    let input_connections = input
        .keys()
        .map(|input| {
            ConnectionDefinition::new(("vino::schematic_input", input), ("component", input))
        })
        .collect();
    let claims = actor.claims();
    let metadata = claims
        .metadata
        .context("Actor has no metadata in its claims, can not find its ports")?;
    let output_ports = metadata
        .outputs
        .context("Actor has no input ports defined in its claims")?;

    let output_connections: Vec<ConnectionDefinition> = output_ports
        .iter()
        .map(|port| {
            ConnectionDefinition::new(("component", port), ("vino::schematic_output", port))
        })
        .collect();

    let connections = itertools::concat(vec![input_connections, output_connections]);
    debug!(
        "Dynamically generated connections: {}",
        itertools::join(&connections, ", ")
    );
    Ok(HostManifest {
        actors: vec![actor_ref],
        labels: HashMap::new(),
        capabilities: vec![],
        links: vec![],
        references: vec![],
        connections: vec![],
        schematics: vec![SchematicDefinition::new(
            "dynamic",
            vec![(
                "component".to_string(),
                ComponentDefinition::new(actor.public_key(), None),
            )],
            connections,
            vec![],
        )],
    })
}
