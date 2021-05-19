pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod options;

use anyhow::Context;

use futures::try_join;
use std::fs::File;
use std::io::prelude::*;
use wasmcloud_host::{HostBuilder, HostManifest, Result};

use crate::options::get_args;
use logger::Logger;

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = get_args();

    Logger::init(&cli).context("Failed to start logger")?;

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
