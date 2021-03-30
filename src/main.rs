use anyhow::Context;
use futures::try_join;
use std::io::prelude::*;
use std::{env::temp_dir, fs::File, path::PathBuf};
use structopt::{clap::AppSettings, StructOpt};
use wasmcloud_host::{HostBuilder, HostManifest, Result};
#[macro_use]
extern crate log;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
     name = "vino")]
struct Cli {
    /// Host for RPC connection
    #[structopt(long = "rpc-host", default_value = "0.0.0.0", env = "VINO_RPC_HOST")]
    rpc_host: String,

    /// Port for RPC connection
    #[structopt(long = "rpc-port", default_value = "4222", env = "VINO_RPC_PORT")]
    rpc_port: String,

    /// JWT file for RPC authentication. Must be supplied with rpc_seed.
    #[structopt(long = "rpc-jwt", env = "VINO_RPC_JWT", hide_env_values = true)]
    rpc_jwt: Option<String>,

    /// Seed file or literal for RPC authentication. Must be supplied with rpc_jwt.
    #[structopt(long = "rpc-seed", env = "VINO_RPC_SEED", hide_env_values = true)]
    rpc_seed: Option<String>,

    /// Credsfile for RPC authentication
    #[structopt(long = "rpc-credsfile", env = "VINO_RPC_CREDS", hide_env_values = true)]
    rpc_credsfile: Option<String>,

    /// JWT file for control interface authentication. Must be supplied with control_seed.
    #[structopt(long = "control-jwt", env = "VINO_CONTROL_JWT", hide_env_values = true)]
    control_jwt: Option<String>,

    /// Seed file or literal for control interface authentication. Must be supplied with control_jwt.
    #[structopt(
        long = "control-seed",
        env = "VINO_CONTROL_SEED",
        hide_env_values = true
    )]
    control_seed: Option<String>,

    /// Credsfile for control interface authentication
    #[structopt(
        long = "control-credsfile",
        env = "VINO_CONTROL_CREDS",
        hide_env_values = true
    )]
    control_credsfile: Option<String>,

    /// Allows live updating of actors
    #[structopt(long = "allow-live-updates")]
    allow_live_updates: bool,

    /// Allows the use of "latest" artifact tag
    #[structopt(long = "allow-oci-latest")]
    allow_oci_latest: bool,

    /// Disables strict comparison of live updated actor claims
    #[structopt(long = "disable-strict-update-check")]
    disable_strict_update_check: bool,

    /// Allows the use of HTTP registry connections to these registries
    #[structopt(long = "allowed-insecure")]
    allowed_insecure: Vec<String>,

    /// Specifies a manifest file to apply to the host once started
    #[structopt(long = "manifest", short = "m", parse(from_os_str))]
    manifest: Option<PathBuf>,

    /// Actor to call
    #[structopt()]
    actor: Option<String>,

    /// Actor operation name
    #[structopt()]
    command: Option<String>,

    /// JSON data
    #[structopt()]
    data: Option<String>,
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = Cli::from_args();
    pretty_env_logger::init();

    if let Some(ref manifest_file) = cli.manifest {
        if !manifest_file.exists() {
            error!(
                "Specified manifest file {:?} could not be opened",
                manifest_file
            );
            return Err("Manifest file could not be opened.".into());
        }
    }

    debug!("Attempting connection to NATS server");
    let nats_url = &format!("{}:{}", cli.rpc_host, cli.rpc_port);
    let nc_rpc = nats_connection(nats_url, cli.rpc_jwt, cli.rpc_seed, cli.rpc_credsfile);
    let nc_control = nats_connection(
        nats_url,
        cli.control_jwt,
        cli.control_seed,
        cli.control_credsfile,
    );

    let mut host_builder = HostBuilder::new();

    match try_join!(nc_rpc, nc_control) {
        Ok((nc_rpc, nc_control)) => {
            host_builder = host_builder
                .with_rpc_client(nc_rpc)
                .with_control_client(nc_control);
        }
        Err(e) => warn!("Could not connect to NATS, operating locally ({})", e),
    }

    if cli.allow_live_updates {
        debug!("Enabling live updates");
        host_builder = host_builder.enable_live_updates();
    }
    if cli.allow_oci_latest {
        debug!("Enabling :latest tag");
        host_builder = host_builder.oci_allow_latest();
    }
    if cli.disable_strict_update_check {
        debug!("Disabling strict update checks");
        host_builder = host_builder.disable_strict_update_check();
    }
    if !cli.allowed_insecure.is_empty() {
        host_builder = host_builder.oci_allow_insecure(cli.allowed_insecure);
    }

    let host = host_builder.build();

    debug!("Starting host");
    match host.start().await {
        Ok(_) => {
            if let Some(pb) = cli.manifest {
                debug!("Applying manifest");
                if pb.exists() {
                    let hm = HostManifest::from_path(pb, true)?;
                    host.apply_manifest(hm).await?;
                    info!("Manifest applied");
                } else {
                    error!("No file exists at location {}", pb.to_string_lossy());
                }
            }
        }
        Err(e) => {
            error!("Failed to start host: {}", e);
        }
    }

    if let (Some(actor), Some(command)) = (cli.actor, cli.command) {
        let data = cli.data.unwrap_or_else(|| "".to_string());
        let actor_pb = PathBuf::from(&actor);
        let actor_pb = if actor_pb.exists() {
            actor_pb
        } else {
            oci_cache_path(&actor)
        };
        match wasmcloud_host::Actor::from_file(actor_pb) {
            Ok(actor) => {
                let key = actor.public_key();
                let json: serde_json::value::Value =
                    serde_json::from_str(&data).context(format!(
                        "Failed to parse input data from the command line data:\n{}",
                        data
                    ))?;
                let messagebytes =
                    serdeconv::to_msgpack_vec(&json).context("Failed to convert to msgpack")?;
                let result = host.call_actor(&key, &command, &messagebytes).await?;
                let parser_result =
                    serdeconv::from_msgpack_slice::<serde_json::value::Value>(&result)
                        .context("failed to deserialize actor return data")?;
                let result = serde_json::to_string(&parser_result).context(format!(
                    "failed to parse return value as JSON: {}",
                    parser_result
                ))?;
                println!("{}", result);
            }
            Err(e) => error!("Error calling actor {}", e),
        }
    }

    actix_rt::signal::ctrl_c().await.unwrap();
    info!("Ctrl-C received, shutting down");
    host.stop().await;
    Ok(())
}

async fn nats_connection(
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
fn extract_arg_value(arg: &str) -> Result<String> {
    match File::open(arg) {
        Ok(mut f) => {
            let mut value = String::new();
            f.read_to_string(&mut value)?;
            Ok(value)
        }
        Err(_) => Ok(arg.to_string()),
    }
}

fn oci_cache_path(img: &str) -> PathBuf {
    let path = temp_dir();
    let path = path.join("wasmcloud_ocicache");
    let _ = ::std::fs::create_dir_all(&path);
    // should produce a file like wasmcloud_azurecr_io_kvcounter_v1.bin
    let img = img.replace(":", "_");
    let img = img.replace("/", "_");
    let img = img.replace(".", "_");
    let mut path = path.join(img);
    path.set_extension("bin");

    path.into()
}
