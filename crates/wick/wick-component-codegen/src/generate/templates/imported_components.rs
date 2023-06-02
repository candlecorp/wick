use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::{BoundInterface, OperationSignature};
use wick_interface_types::Field;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::{expand_field_types, expand_input_fields, fields_to_tuples};
use crate::generate::ids::*;
use crate::generate::templates::op_config;
use crate::generate::{f, Direction};
use crate::*;

pub(crate) fn imported_components(config: &mut Config, required: Vec<BoundInterface>) -> TokenStream {
  let fields = required
    .into_iter()
    .map(|v| {
      let name = id(&format!("{}Component", &pascal(v.id())));
      let configs = named_configs(config, v.kind().operations());
      let ops = operation_impls(config, v.kind().operations());

      config.add_dep(Dependency::WickPacket);
      quote! {
        pub struct #name {
          component: wick_packet::ComponentReference,
        }
        #(#configs)*
        impl #name {
          pub fn new(component: wick_packet::ComponentReference) -> Self {
            Self { component }
          }
          #[allow(unused)]
          pub fn component(&self) -> &wick_packet::ComponentReference {
            &self.component
          }
          #(#ops)*
        }
      }
    })
    .collect_vec();
  quote! {
      #(#fields),*

  }
}

fn operation_impls(config: &mut Config, ops: &HashMap<String, OperationSignature>) -> Vec<TokenStream> {
  let dir = Direction::In;
  let raw = false;
  ops
    .values()
    .map(|op| {
      config.add_dep(Dependency::WickComponent);

      let op_name = op.name();
      let name = id(&snake(op_name));
      let (op_config_id, set_context) = if !op.config().is_empty() {
        let id = id(&named_config_id(op.name()));
        (quote! { op_config: #id, }, quote! { Some(op_config.into()) })
      } else {
        (quote! {}, quote! { None })
      };
      let inputs = expand_input_fields(config, op.inputs(), dir, raw);
      let encode_inputs = encoded_inputs(op.inputs());
      let merge_inputs = merged_inputs(op.inputs());
      let response_stream_types = expand_field_types(config, op.outputs(), dir, raw);
      let fan_out: Vec<_> = fields_to_tuples(config, op.outputs(), dir, raw);
      let types = f::maybe_parens(response_stream_types);

      quote! {
        pub fn #name(&self, #op_config_id #(#inputs),*) -> std::result::Result<#types,wick_packet::Error> {
          use wick_component::StreamExt;
          #(#encode_inputs)*
          let stream = wick_component::empty();
          let stream = #merge_inputs;
          let stream = wick_packet::PacketStream::new(Box::pin(stream));

          let mut stream = self.component.call(#op_name, stream, #set_context)?;
          Ok(wick_component::payload_fan_out!(stream, raw: false, wick_component::BoxError, [#(#fan_out),*]))
        }
      }
    })
    .collect_vec()
}

fn named_configs(config: &mut Config, ops: &HashMap<String, OperationSignature>) -> Vec<TokenStream> {
  ops
    .values()
    .filter_map(|op| {
      if op.config().is_empty() {
        None
      } else {
        let config_name = named_config_id(op.name());
        let config_id = id(&config_name);
        let def = op_config(config, &config_name, op);
        Some(quote! {
          #def

          impl From<#config_id> for wick_packet::GenericConfig {
            fn from(v: #config_id) -> Self {
              wick_component::to_value(v).unwrap().try_into().unwrap()
            }
          }
        })
      }
    })
    .collect_vec()
}

fn encoded_inputs(fields: &[Field]) -> Vec<TokenStream> {
  fields
    .iter()
    .map(|i| {
      let name = i.name();
      let id = id(&snake(i.name()));
      quote! {
        let #id = #id.map(wick_packet::into_packet(#name));
      }
    })
    .collect_vec()
}

fn merged_inputs(fields: &[Field]) -> TokenStream {
  let start = id("stream");

  fields.iter().fold(quote! {#start}, |acc: TokenStream, next| {
    let name = id(&snake(next.name()));
    quote! {
      #acc.merge(#name)
    }
  })
}
