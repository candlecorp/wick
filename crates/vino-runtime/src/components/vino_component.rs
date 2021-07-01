use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use async_trait::async_trait;
use wascap::jwt::{
  Claims,
  Token,
};

use super::wapc_provider::WapcProvider;
use crate::dev::prelude::*;
use crate::error::ComponentError;
use crate::util::oci::fetch_oci_bytes;

type Result<T> = std::result::Result<T, ComponentError>;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WapcComponent {
  pub(crate) token: Token<wascap::jwt::Actor>,
  pub(crate) bytes: Vec<u8>,
  #[derivative(Debug = "ignore")]
  pub(crate) addr: Option<Addr<WapcProvider>>,
}

#[derive(Clone, Debug)]
pub(crate) enum ComponentType {
  Native,
  WaPC,
}

pub(crate) trait VinoComponent: VinoComponentClone {
  fn public_key(&self) -> String;
  fn id(&self) -> String;
  fn get_inputs(&self) -> Vec<String>;
  fn get_outputs(&self) -> Vec<String>;
  fn name(&self) -> String;
  fn get_kind(&self) -> ComponentType;
  fn claims(&self) -> Claims<wascap::jwt::Actor>;
}

pub(crate) type BoxedComponent = Box<dyn VinoComponent>;
pub(crate) trait VinoComponentClone {
  fn clone_box(&self) -> Box<dyn VinoComponent>;
}

impl<T: 'static + VinoComponent + Clone> VinoComponentClone for T {
  fn clone_box(&self) -> Box<dyn VinoComponent> {
    Box::new(self.clone())
  }
}

impl Clone for BoxedComponent {
  fn clone(&self) -> Self {
    self.clone_box()
  }
}

impl WapcComponent {
  /// Create an actor from the bytes of a signed WebAssembly module. Attempting to load
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub fn from_slice(buf: &[u8]) -> Result<WapcComponent> {
    let token = wascap::wasm::extract_claims(&buf)?;
    token.map_or(Err(ComponentError::ClaimsError), |t| {
      Ok(WapcComponent {
        token: t,
        bytes: buf.to_vec(),
        addr: None,
      })
    })
  }

  /// Create an actor from a signed WebAssembly (`.wasm`) file.
  pub fn from_file(path: &Path) -> Result<WapcComponent> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    WapcComponent::from_slice(&buf).map_err(|_| ComponentError::FileNotFound(path.to_path_buf()))
  }

  /// Obtain the issuer's public key as it resides in the actor's token (the `iss` field of the JWT).
  pub fn _issuer(&self) -> String {
    self.token.claims.issuer.clone()
  }

  /// Obtain the list of capabilities declared in this actor's embedded token.
  pub fn _capabilities(&self) -> Vec<String> {
    match self.token.claims.metadata.as_ref().unwrap().caps {
      Some(ref caps) => caps.clone(),
      None => vec![],
    }
  }

  /// Obtain the list of tags in the actor's token.
  pub fn _tags(&self) -> Vec<String> {
    match self.token.claims.metadata.as_ref().unwrap().tags {
      Some(ref tags) => tags.clone(),
      None => vec![],
    }
  }
}

#[async_trait]
impl VinoComponent for WapcComponent {
  /// Obtain the actor's public key (The `sub` field of the JWT).
  fn public_key(&self) -> String {
    self.token.claims.subject.clone()
  }

  /// A globally referencable ID to this component
  fn id(&self) -> String {
    self.public_key()
  }

  fn get_inputs(&self) -> Vec<String> {
    match self.token.claims.metadata.as_ref().unwrap().inputs {
      Some(ref n) => n.clone(),
      None => vec![],
    }
  }

  fn get_outputs(&self) -> Vec<String> {
    match self.token.claims.metadata.as_ref().unwrap().outputs {
      Some(ref n) => n.clone(),
      None => vec![],
    }
  }

  /// The actor's human-friendly display name
  fn name(&self) -> String {
    match self.token.claims.metadata.as_ref().unwrap().name {
      Some(ref n) => n.clone(),
      None => "Unnamed".to_owned(),
    }
  }

  fn get_kind(&self) -> ComponentType {
    ComponentType::WaPC
  }

  // Obtain the raw set of claims for this actor.
  fn claims(&self) -> Claims<wascap::jwt::Actor> {
    self.token.claims.clone()
  }
}

#[derive(Clone, Debug)]
pub(crate) struct NativeComponent {
  pub(crate) namespace: String,
  pub(crate) id: String,
  pub(crate) inputs: Vec<String>,
  pub(crate) outputs: Vec<String>,
}

impl NativeComponent {}

#[async_trait]
impl VinoComponent for NativeComponent {
  fn public_key(&self) -> String {
    panic!()
  }

  fn id(&self) -> String {
    format!("{}::{}", self.namespace, self.id)
  }

  fn get_inputs(&self) -> Vec<String> {
    self.inputs.clone()
  }

  fn get_outputs(&self) -> Vec<String> {
    self.outputs.clone()
  }

  /// The actor's human-friendly display name
  fn name(&self) -> String {
    self.id.clone()
  }

  fn get_kind(&self) -> ComponentType {
    ComponentType::Native
  }

  // Obtain the raw set of claims for this actor.
  fn claims(&self) -> Claims<wascap::jwt::Actor> {
    Claims::default()
  }
}

async fn start_wapc_actor_from_file(p: &Path) -> Result<WapcComponent> {
  let component = WapcComponent::from_file(p)?;
  trace!(
    "Starting wapc component '{}' from file {}",
    component.name(),
    p.to_string_lossy()
  );
  Ok(component)
}

async fn start_wapc_actor_from_oci(
  url: &str,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcComponent> {
  let bytes = fetch_oci_bytes(url, allow_latest, allowed_insecure).await?;
  let component = WapcComponent::from_slice(&bytes)?;

  trace!(
    "Starting wapc component '{}' from URL {}",
    component.name(),
    url
  );

  Ok(component)
}

pub(crate) async fn load_component(
  comp_ref: String,
  allow_latest: bool,
  allowed_insecure: &[String],
) -> Result<WapcComponent> {
  let p = Path::new(&comp_ref);
  let component = if p.exists() {
    debug!("{:?} exists on file system, loading from disk", p);
    start_wapc_actor_from_file(p).await
  } else {
    debug!(
      "{:?} does not exist on file system, trying as OCI url",
      comp_ref
    );
    start_wapc_actor_from_oci(&comp_ref, allow_latest, allowed_insecure).await
  };
  component
}
