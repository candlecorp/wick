use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::{OperationSignature, StructDefinition, TypeDefinition};

use crate::generate::config;
use crate::generate::dependency::Dependency;
use crate::generate::templates::TypeOptions;

pub(crate) fn op_config(config: &mut config::Config, config_name: &str, op: &OperationSignature) -> TokenStream {
  let (_, config_def) = super::type_def(
    config,
    &TypeDefinition::Struct(StructDefinition::new(config_name, op.config().to_vec(), None)),
    TypeOptions::Defaults,
  );

  let config = config.output_structs.then(|| {
    config.add_dep(Dependency::WickPacket);
    config.add_dep(Dependency::WasmRsRx);
    config.add_dep(Dependency::WasmRs);
    quote! {
      #config_def

      impl From<Config> for wick_packet::RuntimeConfig {
        fn from(v: Config) -> Self {
          wick_component::to_value(v).unwrap().try_into().unwrap()
        }
      }
    }
  });

  quote! {
    #config
  }
}
