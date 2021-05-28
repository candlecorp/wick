use crate::{error::VinoError, oci::fetch_oci_bytes, Result};
use anyhow::Context;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use vino_runtime::run_config::RunConfig;
use wasmcloud_host::vino::{ComponentDefinition, ConnectionDefinition, SchematicDefinition};
use wasmcloud_host::{Actor, HostManifest};

pub fn generate_exec_manifest(
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

pub(crate) async fn nats_connection(
    url: &str,
    jwt: Option<String>,
    seed: Option<String>,
    credsfile: Option<String>,
) -> Result<nats::asynk::Connection> {
    if let (Some(jwt_file), Some(seed_val)) = (jwt, seed) {
        let kp = nkeys::KeyPair::from_seed(&extract_arg_value(&seed_val)?)?;
        // You must provide the JWT via a closure
        Ok(nats::Options::with_jwt(
            move || Ok(jwt_file.clone()),
            move |nonce| kp.sign(nonce).unwrap(),
        )
        .connect_async(url)
        .await?)
    } else if let Some(credsfile_path) = credsfile {
        Ok(nats::Options::with_credentials(credsfile_path)
            .connect_async(url)
            .await?)
    } else {
        Ok(nats::asynk::connect(url).await?)
    }
}

/// Returns value from an argument that may be a file path or the value itself
pub(crate) fn extract_arg_value(arg: &str) -> Result<String> {
    match File::open(arg) {
        Ok(mut f) => {
            let mut value = String::new();
            f.read_to_string(&mut value)?;
            Ok(value)
        }
        Err(_) => Ok(arg.to_string()),
    }
}

pub(crate) async fn fetch_actor(
    actor_ref: String,
    allow_latest: bool,
    allowed_insecure: Vec<String>,
) -> Result<Actor> {
    let p = Path::new(&actor_ref);
    if p.exists() {
        Ok(wasmcloud_host::Actor::from_file(p)?)
    } else {
        let actor_bytes = fetch_oci_bytes(&actor_ref, allow_latest, &allowed_insecure).await?;
        Ok(wasmcloud_host::Actor::from_slice(&actor_bytes)?)
    }
}

pub fn load_runconfig(path: PathBuf) -> Result<RunConfig> {
    trace!("Loading configuration from {}", path.to_string_lossy());
    let mut file = File::open(path.clone())
        .map_err(|_| VinoError::FileNotFound(path.to_string_lossy().into()))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_runconfig(buf)
}

pub fn parse_runconfig(src: String) -> Result<RunConfig> {
    serde_yaml::from_slice::<RunConfig>(src.as_bytes())
        .map_err(|e| VinoError::ConfigurationDeserialization(e.to_string()))
}
