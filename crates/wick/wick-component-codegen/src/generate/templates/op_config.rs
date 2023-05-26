use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::OperationSignature;
use wick_interface_types::{StructSignature, TypeDefinition};

use crate::generate::dependency::Dependency;
use crate::generate::ids::*;
use crate::generate::templates::TypeOptions;
use crate::generate::{config, f};

pub(crate) fn op_config(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let config_name = config_id(op.name());
  let (_, config_def) = super::type_def(
    config,
    &TypeDefinition::Struct(StructSignature::new(config_name, op.config().to_vec())),
    TypeOptions::Defaults,
  );

  let outputs = f::gen_if(
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
    #outputs
  }
}
