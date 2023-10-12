use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::Field;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, Direction};

pub(crate) fn op_outgoing(config: &mut config::Config, name: &str, fields: &[Field]) -> TokenStream {
  let outgoing_name = id(name);

  let output_port_fields = fields
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::Out, false, config.raw, &i.ty);
      quote! {pub(crate) #port_field_name: wick_packet::OutgoingPort<#port_type>}
    })
    .collect_vec();

  let output_port_fields_mut = fields
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      quote! {&mut self.#port_field_name}
    })
    .collect_vec();

  let output_ports_new = fields
    .iter()
    .map(|out| {
      let port_name = &out.name;
      let port_field_name = id(&snake(&out.name));
      quote! {#port_field_name: wick_packet::OutgoingPort::new(#port_name, channel.clone())}
    })
    .collect_vec();

  let single_output_impl = (fields.len() == 1).then(|| {
    let output = fields.first().unwrap();
    let name = id(&snake(output.name()));
    quote! {
      impl wick_component::SingleOutput for #outgoing_name {
        fn single_output(&mut self) -> &mut dyn wick_packet::Port {
          &mut self.#name
        }
      }
    }
  });

  let outputs = config.output_structs.then(|| {
    config.add_dep(Dependency::WickPacket);
    config.add_dep(Dependency::WasmRsRx);
    config.add_dep(Dependency::WasmRs);

    quote! {
      pub struct #outgoing_name {
        pub(crate) channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>,
        #[allow(unused)]
        #(#output_port_fields),*
      }

      impl wick_component::Broadcast for #outgoing_name {
        fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_>{
          wick_packet::OutputIterator::new(vec![#(#output_port_fields_mut),*])
        }
      }

      impl wick_packet::WasmRsChannel for #outgoing_name {
        fn channel(&self) -> wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError> {
          self.channel.clone()
        }
      }

      #single_output_impl

      impl #outgoing_name {
        #[allow(unused)]
        pub fn new() -> Self {
          let channel = wasmrs_rx::FluxChannel::new();
          Self {
            #(#output_ports_new,)*
            channel,
          }
        }
        #[allow(unused)]
        pub fn with_channel(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
          Self {
            #(#output_ports_new,)*
            channel,
          }
        }
      }
    }
  });
  quote! {#outputs}
}

pub(crate) fn op_simple_outgoing(config: &mut config::Config, name: &str, into: &str, fields: &[Field]) -> TokenStream {
  let outgoing_name = id(name);
  let impl_into_name = id(into);

  let output_port_fields = fields
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::Out, false, false, &i.ty);
      quote! {pub(crate) #port_field_name: #port_type}
    })
    .collect_vec();

  let port_send = fields
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      quote! {inputs.#port_field_name.send(v.#port_field_name);}
    })
    .collect_vec();

  let outputs = config.output_structs.then(|| {
    config.add_dep(Dependency::WickPacket);
    config.add_dep(Dependency::WasmRsRx);
    config.add_dep(Dependency::WasmRs);

    quote! {
      pub struct #outgoing_name {
        #[allow(unused)]
        #(#output_port_fields),*
      }
      impl From<#outgoing_name> for #impl_into_name{
        fn from(v:#outgoing_name) -> #impl_into_name {
          let mut inputs =  #impl_into_name::new();
          #(#port_send)*
          inputs
        }
      }

    }
  });
  quote! {#outputs}
}
