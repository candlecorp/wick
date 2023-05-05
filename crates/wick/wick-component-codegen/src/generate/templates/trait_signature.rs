use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::OperationSignature;
use wick_interface_types::{StructSignature, TypeDefinition};

use crate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::templates::TypeOptions;
use crate::generate::{config, f, Direction};

pub(crate) fn trait_signature(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let outputs_name = id(&op_outputs_name(op));
  let trait_name = id(&format!("Op{}", &pascal(op.name())));
  let impl_name = id(&snake(op.name()));
  let output_ports = op
    .outputs()
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::Out, false, &i.ty);
      quote! {pub(crate) #port_field_name: wick_packet::Output<#port_type>}
    })
    .collect_vec();
  let output_ports_new = op
    .outputs()
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = id(&snake(&i.name));
      quote! {#port_field_name: wick_packet::Output::new(#port_name, channel.clone())}
    })
    .collect_vec();

  let inputs = op
    .inputs()
    .iter()
    .map(|i| {
      let port_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::In, false, &i.ty);
      quote! {#port_name: WickStream<#port_type>}
    })
    .collect_vec();
  let config_name = config_id(op.name());
  let (_, config_def) = super::type_def(
    config,
    &TypeDefinition::Struct(StructSignature::new(&config_name, op.config().to_vec())),
    TypeOptions::Defaults,
  );

  let config_id = id(&config_name);

  let traits = f::gen_if(
    config.op_traits,
    || {
      config.add_dep(Dependency::AsyncTrait);
    },
    quote! {
      #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
      #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
      pub trait #trait_name {
        #[allow(unused)]
        async fn #impl_name(#(#inputs,)* outputs: #outputs_name, ctx: wick_component::flow_component::Context<#config_id>) -> Result<()> {unimplemented!()}
      }
    },
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

    pub struct #outputs_name {
      #[allow(unused)]
      #(#output_ports,)*
    }
    impl #outputs_name {
      pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
        Self {
          #(#output_ports_new,)*
        }
      }
    }},
  );

  quote! {
    #outputs
    #traits
  }
}
