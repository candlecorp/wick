use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::Field;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, Direction};
#[allow(clippy::too_many_lines)]
pub(crate) fn op_incoming(config: &mut config::Config, name: &str, fields: &[Field]) -> TokenStream {
  let inputs_name = id(name);

  let input_port_fields = fields
    .iter()
    .map(|i| {
      let port_field_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::In, false, false, &i.ty);

      quote! {#port_field_name: BoxStream<VPacket<#port_type>>}
    })
    .collect_vec();

  let input_pairs = fields
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(config, Direction::In, false, false, &i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect_vec();
  let config_id = id(&generic_config_id());
  let inputs = fields.iter().map(|i| id(&snake(&i.name))).collect_vec();

  let unary = (fields.len() == 1).then(|| {
    let input = fields.first().unwrap();
    let name = id(&snake(input.name()));
    let port_type = expand_type(config, Direction::In, false, false, &input.ty);
    quote! {
      impl wick_packet::UnaryInputs<#port_type> for #inputs_name {
        fn input(&mut self) -> &mut BoxStream<VPacket<#port_type>> {
          &mut self.#name
        }
        fn take_input( self) ->BoxStream<VPacket<#port_type>> {
          self.#name
        }

      }
    }
  });

  let binary = (fields.len() == 2).then(|| {
    let left = fields.first().unwrap();
    let left_id = id(&snake(left.name()));
    let left_type = expand_type(config, Direction::In, false, false, &left.ty);

    let right = &fields[1];
    let right_id = id(&snake(right.name()));
    let right_type = expand_type(config, Direction::In, false, false, &right.ty);

    quote! {
      impl wick_packet::BinaryInputs<#left_type,#right_type> for #inputs_name {
        fn left(&mut self) -> &mut BoxStream<VPacket<#left_type>> {
          &mut self.#left_id
        }
        fn right(&mut self) -> &mut BoxStream<VPacket<#right_type>> {
          &mut self.#right_id
        }
        fn both(self) -> (BoxStream<VPacket<#left_type>>,BoxStream<VPacket<#right_type>>) {
          let Self{#left_id, #right_id} = self;
          (#left_id,#right_id)
        }
      }
    }
  });

  let input_struct =  config.output_structs.then(||
    {
      config.add_dep(Dependency::WickPacket);
      config.add_dep(Dependency::WasmRsRx);
      config.add_dep(Dependency::WasmRs);
      let process = quote!{
          pub fn process_incoming(mut stream: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>) -> (wasmrs_rx::BoxMono<Context<#config_id>,String>, #inputs_name) {
            #[allow(unused_parens)]
            let (config, (#(#inputs),*)) = wick_component::payload_fan_out!(stream, wick_component::AnyError, #config_id, [#(#input_pairs),*]);

            (config,#inputs_name::new(#(#inputs),*))
          }
        };
      quote! {
        pub struct #inputs_name {
          #(pub(crate) #input_port_fields,)*
        }

        #unary

        #binary

        #process

        impl #inputs_name {
          pub fn new(#(#input_port_fields),*) -> Self {
            Self {
              #(#inputs),*
            }
          }
        }
      }
    }
  );

  quote! {
    #input_struct
  }
}
