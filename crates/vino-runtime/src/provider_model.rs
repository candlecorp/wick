use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::components::vino_component::{
  load_component,
  VinoComponent,
};
use crate::components::{
  grpc_url_provider,
  native_provider,
  wapc_provider,
};
use crate::dev::prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct ProviderModel {
  pub(crate) namespace: String,
  pub(crate) components: HashMap<String, ComponentModel>,
}

#[derive(Debug)]
pub(crate) struct ProviderChannel {
  pub(crate) namespace: String,
  pub(crate) arbiter: Arbiter,
  pub(crate) recipient: Recipient<Invocation>,
}

pub(crate) async fn initialize_provider(
  provider: ProviderDefinition,
  seed: &str,
  allow_lastest: bool,
  allowed_insecure: &[String],
) -> Result<(ProviderChannel, ProviderModel)> {
  let arbiter = Arbiter::new();
  let namespace = provider.namespace.clone();
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
          namespace: namespace.clone(),
        })
        .await??;

      (
        ProviderChannel {
          namespace: namespace.clone(),
          arbiter,
          recipient: addr.recipient(),
        },
        ProviderModel {
          namespace,
          components,
        },
      )
    }
    ProviderKind::GrpcUrl => {
      let addr = grpc_url_provider::GrpcUrlProvider::start_in_arbiter(&handle, |_| {
        grpc_url_provider::GrpcUrlProvider::default()
      });

      let components = addr
        .send(grpc_url_provider::Initialize {
          namespace: namespace.clone(),
          address: provider.reference.clone(),
          signing_seed: seed.to_owned(),
        })
        .await??;

      (
        ProviderChannel {
          namespace: namespace.clone(),
          arbiter,
          recipient: addr.recipient(),
        },
        ProviderModel {
          namespace,
          components,
        },
      )
    }
    ProviderKind::Wapc => {
      let addr = wapc_provider::WapcProvider::start_in_arbiter(&handle, |_| {
        wapc_provider::WapcProvider::default()
      });

      let component =
        load_component(provider.reference.clone(), allow_lastest, allowed_insecure).await?;

      let components = addr
        .send(wapc_provider::Initialize {
          namespace: namespace.clone(),
          signing_seed: seed.to_owned(),
          name: component.name(),
          outputs: component.get_outputs(),
          inputs: component.get_inputs(),
          bytes: component.bytes,
        })
        .await??;

      (
        ProviderChannel {
          namespace: namespace.clone(),
          arbiter,
          recipient: addr.recipient(),
        },
        ProviderModel {
          namespace,
          components,
        },
      )
    }
  };
  Ok(handler)
}
