use std::path::PathBuf;
use std::str::FromStr;

use url::Url;

use crate::config::{
  ConfigOrDefinition,
  ConfigurationTreeNode,
  ResourceBinding,
  ResourceDefinition,
  TcpPort,
  UdpPort,
  UrlResource,
  Volume,
};

/// An audit report for a component or application.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Audit {
  /// The name of the audited element.
  pub name: String,
  /// The resources used by the audited element.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<AuditedResourceBinding>,
  /// The components the audited element imports.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub imports: Vec<Audit>,
}

impl Audit {
  /// Audit a configuration tree.
  pub fn new(tree: &ConfigurationTreeNode) -> Self {
    Self {
      name: tree.name.clone(),
      resources: tree
        .element
        .resources()
        .iter()
        .map(AuditedResourceBinding::from)
        .collect::<Vec<_>>(),
      imports: tree.children.iter().map(Self::config_or_def).collect::<Vec<_>>(),
    }
  }

  /// Audit a flattened list of configuration elements.
  pub fn new_flattened(elements: &[ConfigOrDefinition]) -> Vec<Audit> {
    elements.iter().map(Self::config_or_def).collect::<Vec<_>>()
  }

  pub(crate) fn config_or_def(el: &ConfigOrDefinition) -> Self {
    match el {
      crate::config::ConfigOrDefinition::Config(c) => Audit::new(c),
      crate::config::ConfigOrDefinition::Definition { id, .. } => Audit {
        name: id.clone(),
        resources: Vec::new(),
        imports: Vec::new(),
      },
    }
  }
}

impl From<&ResourceDefinition> for AuditedResource {
  fn from(value: &ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::TcpPort(v) => Self::TcpPort(AuditedPort {
        port: *v.port.value_unchecked(),
        address: v.host.value_unchecked().clone(),
      }),
      ResourceDefinition::UdpPort(v) => Self::UdpPort(AuditedPort {
        port: *v.port.value_unchecked(),
        address: v.host.value_unchecked().clone(),
      }),
      ResourceDefinition::Url(v) => Self::Url(AuditedUrl::from(v.url.value_unchecked().clone())),
      ResourceDefinition::Volume(v) => Self::Volume(AuditedVolume {
        path: v.path().unwrap(),
      }),
    }
  }
}

/// A rendeder resource binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AuditedResourceBinding {
  pub(crate) name: String,
  pub(crate) resource: AuditedResource,
}

impl From<AuditedResourceBinding> for ResourceDefinition {
  fn from(value: AuditedResourceBinding) -> Self {
    match value.resource {
      AuditedResource::TcpPort(v) => Self::TcpPort(TcpPort::new(v.address, v.port)),
      AuditedResource::UdpPort(v) => Self::UdpPort(UdpPort::new(v.address, v.port)),
      AuditedResource::Url(v) => Self::Url(UrlResource::new(v.url)),
      AuditedResource::Volume(v) => Self::Volume(Volume::new(v.path.to_string_lossy().to_string())),
    }
  }
}

impl std::fmt::Display for AuditedResourceBinding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.name, self.resource)
  }
}

impl From<&ResourceBinding> for AuditedResourceBinding {
  fn from(value: &ResourceBinding) -> Self {
    Self {
      name: value.id.clone(),
      resource: AuditedResource::from(&value.kind),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(tag = "kind")]
/// The possible types of resources. Resources are system-level resources and sensitive configuration.
pub enum AuditedResource {
  /// A variant representing a [crate::config::TcpPort] type.
  #[serde(rename = "wick/resource/tcpport@v1")]
  TcpPort(AuditedPort),
  /// A variant representing a [crate::config::UdpPort] type.
  #[serde(rename = "wick/resource/udpport@v1")]
  UdpPort(AuditedPort),
  /// A variant representing a [crate::config::UrlResource] type.
  #[serde(rename = "wick/resource/url@v1")]
  Url(AuditedUrl),
  /// A variant representing a [crate::config::Volume] type.
  #[serde(rename = "wick/resource/volume@v1")]
  Volume(AuditedVolume),
}

impl std::fmt::Display for AuditedResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AuditedResource::TcpPort(v) => v.fmt(f),
      AuditedResource::UdpPort(v) => v.fmt(f),
      AuditedResource::Url(v) => v.fmt(f),
      AuditedResource::Volume(v) => v.fmt(f),
    }
  }
}

/// A summary of a UDP port resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AuditedPort {
  pub(crate) port: u16,
  pub(crate) address: String,
}

impl std::fmt::Display for AuditedPort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.address, self.port)
  }
}

/// A summary of a volume resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AuditedVolume {
  pub(crate) path: PathBuf,
}

impl std::fmt::Display for AuditedVolume {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.path.to_string_lossy())
  }
}

/// A summary of a URL resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct AuditedUrl {
  pub(crate) url: Url,
}

impl From<Url> for AuditedUrl {
  fn from(mut url: Url) -> Self {
    let _ = url.set_username("");
    let _ = url.set_password(None);
    Self { url }
  }
}

impl FromStr for AuditedUrl {
  type Err = url::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut url = Url::parse(s)?;
    let _ = url.set_username("");
    let _ = url.set_password(None);

    Ok(Self { url })
  }
}

impl std::fmt::Display for AuditedUrl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.url.as_str())
  }
}
