use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use actix::{
  Addr,
  SyncArbiter,
};
use async_trait::async_trait;
use wascap::jwt::{
  Claims,
  Token,
};

use super::wapc_component_actor::WapcComponentActor;
use super::{
  Inputs,
  Outputs,
};
use crate::components::native_component_actor::NativeComponentActor;
use crate::{
  native_actors,
  Error,
  Result,
};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WapcComponent {
  pub(crate) token: Token<wascap::jwt::Actor>,
  pub(crate) bytes: Vec<u8>,
  #[derivative(Debug = "ignore")]
  pub(crate) addr: Option<Addr<WapcComponentActor>>,
}

// pub type StartFuture<T> = Pin<Box<dyn Future<Output = Addr<T>> + Send>>;

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

pub(crate) trait Start {
  type Item;

  fn start(&mut self, seed: String) -> Result<Addr<Self::Item>>
  where
    Self::Item: actix::Actor;
}

impl WapcComponent {
  /// Create an actor from the bytes of a signed WebAssembly module. Attempting to load
  /// an unsigned module, or a module signed improperly, will result in an error.
  pub fn from_slice(buf: &[u8]) -> Result<WapcComponent> {
    let token = wascap::wasm::extract_claims(&buf)?;
    if let Some(t) = token {
      Ok(WapcComponent {
        token: t,
        bytes: buf.to_vec(),
        addr: None,
      })
    } else {
      Err("Unable to extract embedded token from WebAssembly module".into())
    }
  }

  /// Create an actor from a signed WebAssembly (`.wasm`) file.
  pub fn from_file(path: impl AsRef<Path>) -> Result<WapcComponent> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    WapcComponent::from_slice(&buf)
  }

  /// Obtain the issuer's public key as it resides in the actor's token (the `iss` field of the JWT).
  pub fn _issuer(&self) -> String {
    self.token.claims.issuer.to_string()
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
    self.token.claims.subject.to_string()
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
      Some(ref n) => n.to_string(),
      None => "Unnamed".to_string(),
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

impl Start for WapcComponent {
  type Item = WapcComponentActor;
  fn start(&mut self, _seed: String) -> Result<Addr<WapcComponentActor>> {
    // let init = crate::wapc_component_actor::Initialize {
    //     actor_bytes: self.bytes.clone(),
    //     signing_seed: seed,
    // };
    let new_actor = SyncArbiter::start(1, WapcComponentActor::default);

    // let new_actor = WapcComponentActor::default().start();
    self.addr = Some(new_actor.clone());
    // new_actor.send(init).await?;
    Ok(new_actor)
  }
}

#[derive(Clone, Debug)]
pub struct NativeComponent {
  pub namespace: String,
  pub id: String,
  pub inputs: Inputs,
  pub outputs: Outputs,
}

impl NativeComponent {
  pub(crate) fn from_id(namespace: String, name: String) -> Result<NativeComponent> {
    match native_actors::get_native_actor(&name) {
      Some(actor) => Ok(NativeComponent {
        namespace,
        id: name,
        inputs: actor.get_input_ports(),
        outputs: actor.get_output_ports(),
      }),
      None => Err(Error::ComponentError(format!(
        "Could not find actor {}",
        name
      ))),
    }
  }
}

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
    self.id.to_string()
  }

  fn get_kind(&self) -> ComponentType {
    ComponentType::Native
  }

  // Obtain the raw set of claims for this actor.
  fn claims(&self) -> Claims<wascap::jwt::Actor> {
    Claims::default()
  }
}

impl Start for NativeComponent {
  type Item = NativeComponentActor;
  fn start(&mut self, _seed: String) -> Result<Addr<NativeComponentActor>> {
    let native_actor = SyncArbiter::start(1, NativeComponentActor::default);
    // native_actor
    //     .send(native_component_actor::Initialize { name: self.name() })
    //     .await?;
    Ok(native_actor)
  }
}
