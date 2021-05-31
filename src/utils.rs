use crate::{
    commands::{HostOptions, NatsOptions},
    error::VinoError,
    oci::fetch_oci_bytes,
    Result,
};
use anyhow::Context;
use logger::LoggingOptions;
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

fn this_or_that_option<T>(a: Option<T>, b: Option<T>) -> Option<T> {
    if a.is_some() {
        a
    } else {
        b
    }
}

pub fn merge_runconfig(base: RunConfig, nats: NatsOptions, host: HostOptions) -> RunConfig {
    RunConfig {
        manifest: base.manifest,
        config: vino_runtime::run_config::CommonConfiguration {
            rpc_host: nats.rpc_host.unwrap_or(base.config.rpc_host),
            rpc_port: nats.rpc_port.unwrap_or(base.config.rpc_port),
            rpc_credsfile: this_or_that_option(nats.rpc_credsfile, base.config.rpc_credsfile),
            rpc_jwt: this_or_that_option(nats.rpc_jwt, base.config.rpc_jwt),
            rpc_seed: this_or_that_option(nats.rpc_seed, base.config.rpc_seed),
            control_host: nats.control_host.unwrap_or(base.config.control_host),
            control_port: nats.control_port.unwrap_or(base.config.control_port),
            control_credsfile: this_or_that_option(
                nats.control_credsfile,
                base.config.control_credsfile,
            ),
            control_jwt: this_or_that_option(nats.control_jwt, base.config.control_jwt),
            control_seed: this_or_that_option(nats.control_seed, base.config.control_seed),
            allow_oci_latest: host
                .allow_oci_latest
                .unwrap_or(base.config.allow_oci_latest),
            allowed_insecure: vec![base.config.allowed_insecure, host.allowed_insecure].concat(),
        },
        default_schematic: base.default_schematic,
    }
}

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
    logger::Logger::init(
        &opts,
        &["logger", "vino", "wasmcloud", "wasmcloud_host", "wapc"],
        &[],
    )
    .context("Could not initialize logger")?;
    Ok(())
}
