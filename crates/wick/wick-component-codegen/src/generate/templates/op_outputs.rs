use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, f, Direction};

pub(crate) fn op_outputs(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let outputs_name = id(&op_outputs_name(op));

  let output_port_fields = op
    .outputs()
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::Out, false, config.raw, &i.ty);
      quote! {pub(crate) #port_field_name: wick_packet::OutgoingPort<#port_type>}
    })
    .collect_vec();

  let output_port_fields_mut = op
    .outputs()
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      quote! {&mut self.#port_field_name}
    })
    .collect_vec();

  let output_ports_new = op
    .outputs()
    .iter()
    .enumerate()
    .map(|(i, out)| {
      let port_name = &out.name;
      let port_field_name = id(&snake(&out.name));
      if i < op.outputs.len() - 1 {
        quote! {#port_field_name: wick_packet::OutgoingPort::new(#port_name, channel.clone())}
      } else {
        quote! {#port_field_name: wick_packet::OutgoingPort::new(#port_name, channel)}
      }
    })
    .collect_vec();

  let single_output_impl = (op.outputs().len() == 1).then(|| {
    let output = op.outputs().first().unwrap();
    let name = id(&snake(output.name()));
    quote! {
      impl wick_component::SingleOutput for #outputs_name {
        fn single_output(&mut self) -> &mut dyn wick_packet::Port {
          &mut self.#name
        }
      }
    }
  });

  let outputs = f::gen_if(
    config.output_structs,
    || {
      config.add_dep(Dependency::WickPacket);
      config.add_dep(Dependency::WasmRsRx);
      config.add_dep(Dependency::WasmRs);
    },
    quote! {
      pub struct #outputs_name {
        #[allow(unused)]
        #(#output_port_fields,)*
      }

      impl wick_component::Broadcast for #outputs_name {
        fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_>{
          wick_packet::OutputIterator::new(vec![#(#output_port_fields_mut),*])
        }
      }
      impl wick_component::Broadcast for #outputs_name {
        fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_>{
          wick_packet::OutputIterator::new(vec![#(#output_port_fields_mut),*])
        }
      }

      #single_output_impl

      impl #outputs_name {
        pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
          Self {
            #(#output_ports_new,)*
          }
        }
      }
    },
  );

  quote! {
    #outputs
  }
}
