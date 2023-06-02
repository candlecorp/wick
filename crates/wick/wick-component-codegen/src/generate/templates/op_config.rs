use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::OperationSignature;
use wick_interface_types::{StructDefinition, TypeDefinition};

use crate::generate::dependency::Dependency;
use crate::generate::templates::TypeOptions;
use crate::generate::{config, f};

pub(crate) fn op_config(config: &mut config::Config, config_name: &str, op: &OperationSignature) -> TokenStream {
  let (_, config_def) = super::type_def(
    config,
    &TypeDefinition::Struct(StructDefinition::new(config_name, op.config().to_vec())),
    TypeOptions::Defaults,
  );

  let config = f::gen_if(
    config.output_structs,
    || {
      config.add_dep(Dependency::WickPacket);
      config.add_dep(Dependency::WasmRsRx);
      config.add_dep(Dependency::WasmRs);
    },
    quote! {
      #config_def
    },
  );

  quote! {
    #config
  }
}
