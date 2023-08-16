use crate::config::TemplateConfig;
use crate::error::ManifestError;
use crate::{config, v1};

type Result<T> = std::result::Result<T, ManifestError>;

impl TryFrom<v1::ResourceRestriction> for config::ResourceRestriction {
  type Error = ManifestError;

  fn try_from(value: v1::ResourceRestriction) -> Result<Self> {
    Ok(match value {
      v1::ResourceRestriction::VolumeRestriction(v) => Self::Volume(v.try_into()?),
      v1::ResourceRestriction::UrlRestriction(v) => Self::Url(v.try_into()?),
      v1::ResourceRestriction::TcpPortRestriction(v) => Self::TcpPort(v.try_into()?),
      v1::ResourceRestriction::UdpPortRestriction(v) => Self::UdpPort(v.try_into()?),
    })
  }
}

impl TryFrom<config::ResourceRestriction> for v1::ResourceRestriction {
  type Error = ManifestError;

  fn try_from(value: config::ResourceRestriction) -> Result<Self> {
    Ok(match value {
      config::ResourceRestriction::Volume(v) => v1::ResourceRestriction::VolumeRestriction(v.try_into()?),
      config::ResourceRestriction::Url(v) => v1::ResourceRestriction::UrlRestriction(v.try_into()?),
      config::ResourceRestriction::TcpPort(v) => v1::ResourceRestriction::TcpPortRestriction(v.try_into()?),
      config::ResourceRestriction::UdpPort(v) => v1::ResourceRestriction::UdpPortRestriction(v.try_into()?),
    })
  }
}

impl TryFrom<v1::VolumeRestriction> for config::VolumeRestriction {
  type Error = ManifestError;

  fn try_from(value: v1::VolumeRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      allow: TemplateConfig::new_template(value.allow),
    })
  }
}

impl TryFrom<config::VolumeRestriction> for v1::VolumeRestriction {
  type Error = ManifestError;

  fn try_from(value: config::VolumeRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      allow: value.allow.unrender()?,
    })
  }
}

impl TryFrom<v1::UrlRestriction> for config::UrlRestriction {
  type Error = ManifestError;

  fn try_from(value: v1::UrlRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      allow: TemplateConfig::new_template(value.allow),
    })
  }
}

impl TryFrom<config::UrlRestriction> for v1::UrlRestriction {
  type Error = ManifestError;

  fn try_from(value: config::UrlRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      allow: value.allow.unrender()?,
    })
  }
}

impl TryFrom<v1::TcpPortRestriction> for config::PortRestriction {
  type Error = ManifestError;

  fn try_from(value: v1::TcpPortRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      address: TemplateConfig::new_template(value.address),
      port: TemplateConfig::new_template(value.port),
    })
  }
}

impl TryFrom<config::PortRestriction> for v1::TcpPortRestriction {
  type Error = ManifestError;

  fn try_from(value: config::PortRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      address: value.address.unrender()?,
      port: value.port.unrender()?,
    })
  }
}

impl TryFrom<v1::UdpPortRestriction> for config::PortRestriction {
  type Error = ManifestError;

  fn try_from(value: v1::UdpPortRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      address: TemplateConfig::new_template(value.address),
      port: TemplateConfig::new_template(value.port),
    })
  }
}

impl TryFrom<config::PortRestriction> for v1::UdpPortRestriction {
  type Error = ManifestError;

  fn try_from(value: config::PortRestriction) -> Result<Self> {
    Ok(Self {
      components: value.components,
      address: value.address.unrender()?,
      port: value.port.unrender()?,
    })
  }
}
