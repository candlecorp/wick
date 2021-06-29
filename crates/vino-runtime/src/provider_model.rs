use std::collections::HashMap;
use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::Mutex;

use crate::component_model::ComponentModel;
use crate::components::{
  grpc_url_provider,
  native_provider,
  ProviderRequest,
};
use crate::prelude::*;
use crate::schematic_definition::{
  ProviderDefinition,
  ProviderKind,
};

#[derive(Debug)]
pub struct ProviderModel {
  pub(crate) arbiter: Arbiter,
  pub namespace: String,
  pub components: HashMap<String, ComponentModel>,
}

pub(crate) async fn initialize_provider(
  provider: &ProviderDefinition,
  seed: String,
) -> Result<ProviderModel> {
  let arbiter = Arbiter::new();
  let namespace = provider.namespace.to_string();
  trace!("Registering provider under the namespace {}", namespace);
  let handle = arbiter.handle();
  let handler = match provider.kind {
    ProviderKind::Native => {
      let provider = Arc::new(Mutex::new(vino_native_provider::Provider::default()));
      let addr = native_provider::NativeProvider::start_in_arbiter(&handle, |_| {
        native_provider::NativeProvider::default()
      });
      let components = addr
        .send(native_provider::Initialize {
          provider: provider.clone(),
          namespace: namespace.to_string(),
        })
        .await??;

      ProviderModel {
        arbiter,
        namespace,
        components,
      }
    }
    ProviderKind::GrpcUrl => {
      let addr = grpc_url_provider::GrpcUrlProvider::start_in_arbiter(&handle, |_| {
        grpc_url_provider::GrpcUrlProvider::default()
      });

      let components = addr
        .send(grpc_url_provider::Initialize {
          namespace: namespace.to_string(),
          address: provider.reference.to_string(),
          signing_seed: seed.to_string(),
        })
        .await??;

      ProviderModel {
        arbiter,
        namespace,
        components,
      }
    }
  };
  Ok(handler)
}
