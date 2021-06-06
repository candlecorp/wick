use crate::{
    commands::{HostOptions, NatsOptions},
    error::VinoError,
    Result,
};
use logger::LoggingOptions;
use std::{fs::File, io::Read, path::PathBuf};
use vino_host::HostManifest;

pub fn load_runconfig(path: PathBuf) -> Result<HostManifest> {
    trace!("Loading configuration from {}", path.to_string_lossy());
    let mut file = File::open(path.clone())
        .map_err(|_| VinoError::FileNotFound(path.to_string_lossy().into()))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_runconfig(buf)
}

pub fn parse_runconfig(src: String) -> Result<HostManifest> {
    serde_yaml::from_slice::<HostManifest>(src.as_bytes())
        .map_err(|e| VinoError::ConfigurationDeserialization(e.to_string()))
}

fn this_or_that_option<T>(a: Option<T>, b: Option<T>) -> Option<T> {
    if a.is_some() {
        a
    } else {
        b
    }
}

pub fn merge_runconfig(base: HostManifest, nats: NatsOptions, host: HostOptions) -> HostManifest {
    HostManifest {
        manifest: base.manifest,
        config: vino_host::manifest::CommonConfiguration {
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
    )?;
    Ok(())
}
