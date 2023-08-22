use option_utils::OptionUtils;
mod lockdown;
mod tests;

use crate::config::{self, test_config, types_config, ComponentConfiguration};
use crate::error::ManifestError;
use crate::utils::VecTryMapInto;
use crate::{v1, Result, WickConfiguration};

impl TryFrom<v1::WickConfig> for WickConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::WickConfig) -> Result<Self> {
    let new = match def {
      v1::WickConfig::AppConfiguration(v) => WickConfiguration::App(v.try_into()?),
      v1::WickConfig::ComponentConfiguration(v) => WickConfiguration::Component(v.try_into()?),
      v1::WickConfig::TypesConfiguration(v) => WickConfiguration::Types(v.try_into()?),
      v1::WickConfig::TestConfiguration(v) => WickConfiguration::Tests(v.try_into()?),
      v1::WickConfig::LockdownConfiguration(v) => WickConfiguration::Lockdown(v.try_into()?),
    };
    Ok(new)
  }
}

impl TryFrom<v1::TestConfiguration> for test_config::TestConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::TestConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      cases: value.cases.try_map_into()?,
      config: value.with.map_into(),
      name: value.name,
      source: None,
      env: Default::default(),
    })
  }
}

impl TryFrom<config::TestConfiguration> for v1::TestConfiguration {
  type Error = ManifestError;

  fn try_from(value: config::TestConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      with: value.config.map_into(),
      cases: value.cases.try_map_into()?,
    })
  }
}

impl TryFrom<v1::TypesConfiguration> for types_config::TypesConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::TypesConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      metadata: value.metadata.try_map_into()?,
      types: value.types.try_map_into()?,
      operations: value.operations.try_map_into()?,
      source: None,
      package: value.package.try_map_into()?,
    })
  }
}

impl TryFrom<types_config::TypesConfiguration> for v1::TypesConfiguration {
  type Error = ManifestError;

  fn try_from(value: types_config::TypesConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      metadata: value.metadata.try_map_into()?,
      types: value.types.try_map_into()?,
      operations: value.operations.try_map_into()?,
      package: value.package.try_map_into()?,
    })
  }
}

impl TryFrom<v1::ComponentConfiguration> for ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: v1::ComponentConfiguration) -> Result<Self> {
    Ok(ComponentConfiguration {
      source: None,
      metadata: def.metadata.try_map_into()?,
      host: def.host.try_map_into()?,
      name: def.name,
      tests: def.tests.try_map_into()?,
      component: def.component.try_into()?,
      types: def.types.try_map_into()?,
      requires: def.requires.try_map_into()?,
      import: def.import.try_map_into()?,
      resources: def.resources.try_map_into()?,
      cached_types: Default::default(),
      type_cache: Default::default(),
      package: def.package.try_map_into()?,
      root_config: Default::default(),
    })
  }
}

impl TryFrom<ComponentConfiguration> for v1::ComponentConfiguration {
  type Error = ManifestError;

  fn try_from(def: ComponentConfiguration) -> Result<Self> {
    Ok(v1::ComponentConfiguration {
      metadata: def.metadata.try_map_into()?,
      host: def.host.try_map_into()?,
      name: def.name,
      requires: def.requires.try_map_into()?,
      import: def.import.try_map_into()?,
      types: def.types.try_map_into()?,
      resources: def.resources.try_map_into()?,
      tests: def.tests.try_map_into()?,
      component: def.component.try_into()?,
      package: def.package.try_map_into()?,
    })
  }
}

impl TryFrom<v1::LockdownConfiguration> for config::LockdownConfiguration {
  type Error = ManifestError;

  fn try_from(value: v1::LockdownConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      resources: value.resources.try_map_into()?,
      metadata: value.metadata.try_map_into()?,
      source: None,
      env: None,
    })
  }
}

impl TryFrom<config::LockdownConfiguration> for v1::LockdownConfiguration {
  type Error = ManifestError;

  fn try_from(value: config::LockdownConfiguration) -> std::result::Result<Self, Self::Error> {
    Ok(Self {
      resources: value.resources.try_map_into()?,
      metadata: value.metadata.try_map_into()?,
    })
  }
}
