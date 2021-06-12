use std::path::PathBuf;

use log::debug;
use vino_manifest::*;

#[test_env_log::test]
fn load_manifest_yaml() -> Result<()> {
    let path = PathBuf::from("./tests/manifests/v0/logger.yaml");
    let manifest = HostManifest::load_from_file(&path)?;

    let HostManifest::V0(manifest) = manifest;
    assert_eq!(manifest.default_schematic, "logger");

    Ok(())
}

#[test_env_log::test]
fn load_bad_manifest_yaml() -> Result<()> {
    let path = PathBuf::from("./tests/manifests/v0/bad-yaml.yaml");
    let manifest = HostManifest::load_from_file(&path);
    if let Err(Error::YamlError(e)) = manifest {
        debug!("{:?}", e);
    } else {
        panic!("Should have failed")
    }

    Ok(())
}

#[test_env_log::test]
fn load_manifest_hocon() -> Result<()> {
    let path = PathBuf::from("./tests/manifests/v0/logger.manifest");
    let manifest = HostManifest::load_from_file(&path)?;

    let HostManifest::V0(manifest) = manifest;
    assert_eq!(manifest.default_schematic, "logger");

    Ok(())
}

#[test_env_log::test]
fn load_bad_manifest_hocon() -> Result<()> {
    let path = PathBuf::from("./tests/manifests/v0/bad-hocon.manifest");
    let manifest = HostManifest::load_from_file(&path);
    if let Err(Error::HoconError(e)) = manifest {
        debug!("{:?}", e);
    } else {
        panic!("Should have failed")
    }

    Ok(())
}
