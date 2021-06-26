use std::collections::HashMap;
use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::Mutex;
use vino_rpc::HostedType;

use crate::component_model::ComponentModel;
use crate::components::native_provider::{
  Initialize,
  NativeProvider,
};
use crate::components::provider_component::ProviderComponent;
use crate::components::{
  ListRequest,
  ProviderMessage,
};
use crate::prelude::*;
use crate::schematic_definition::{
  ProviderDefinition,
  ProviderKind,
};

#[derive(Debug)]
pub(crate) struct ProviderModel {
  pub(crate) arbiter: Arbiter,
  pub(crate) namespace: String,
  pub(crate) addr: Box<Recipient<ProviderMessage>>,
  pub(crate) components: HashMap<String, ComponentModel>,
}

pub(crate) async fn initialize_provider(
  provider: &ProviderDefinition,
  seed: String,
) -> Result<ProviderModel> {
  let arbiter = Arbiter::new();
  let namespace = provider.namespace.to_string();
  trace!("registering provider under the namespace {}", namespace);
  let handle = arbiter.handle();
  let handler = match provider.kind {
    ProviderKind::Native => {
      let provider = Arc::new(Mutex::new(vino_native_provider::Provider::default()));
      let addr = NativeProvider::start_in_arbiter(&handle, |_| NativeProvider::default());
      let seed2 = seed.clone();
      addr
        .send(Initialize {
          provider: provider.clone(),
          name: namespace.to_string(),
        })
        .await??;
      let response = addr.send(ProviderMessage::List(ListRequest {})).await??;
      let mut metadata: HashMap<String, ComponentModel> = HashMap::new();
      for item in response.into_list_response()? {
        match item {
          HostedType::Component(component) => {
            let component_addr =
              ProviderComponent::start_in_arbiter(&handle, |_| ProviderComponent::default());
            component_addr
              .send(crate::components::provider_component::Initialize {
                name: component.name.clone(),
                seed: seed2.clone(),
                provider: provider.clone(),
              })
              .await??;
            metadata.insert(
              component.name.to_string(),
              ComponentModel {
                id: format!("{}::{}", namespace, component.name),
                name: component.name,
                inputs: component.inputs.into_iter().map(|p| p.name).collect(),
                outputs: component.outputs.into_iter().map(|p| p.name).collect(),
                addr: component_addr.recipient(),
              },
            );
          }
          HostedType::Schematic(_) => panic!("Unimplemented"),
        }
      }

      ProviderModel {
        arbiter,
        namespace,
        addr: Box::new(addr.recipient()),
        components: metadata,
      }
    }
    ProviderKind::GrpcUrl => todo!(),
  };
  Ok(handler)
}
