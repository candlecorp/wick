use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use wick_config::config::Binding;
use wick_interface_types::{Field, OperationSignature, OperationSignatures};

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::{expand_field_types, expand_input_fields, fields_to_tuples};
use crate::generate::ids::*;
use crate::generate::templates::op_config;
use crate::generate::{f, Direction};
use crate::*;

struct ComponentCodegen {
  struct_def: TokenStream,
  struct_impl: TokenStream,
  inputs: Vec<Option<TokenStream>>,
  outputs: Vec<Option<TokenStream>>,
}

impl ToTokens for ComponentCodegen {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    for opt in &self.inputs {
      tokens.extend(opt.clone());
    }
    for opt in &self.outputs {
      tokens.extend(opt.clone());
    }
    tokens.extend(self.struct_def.clone());
    tokens.extend(self.struct_impl.clone());
  }
}

pub(crate) fn imported_components<T: OperationSignatures>(
  name: &str,
  config: &mut Config,
  required: &[Binding<T>],
) -> TokenStream {
  let components = required
    .iter()
    .map(|v| {
      let name = id(&format!("{}Component", &pascal(v.id())));
      let configs = named_configs(config, &v.kind().operation_signatures());
      let ops = operation_impls(config, &v.kind().operation_signatures());
      let (mut op_fns, mut op_inputs, mut op_outputs) = (Vec::new(), Vec::new(), Vec::new());
      for op in ops {
        op_fns.push(op.impls);
        op_inputs.push(op.input);
        op_outputs.push(op.output);
      }

      config.add_dep(Dependency::WickPacket);
      ComponentCodegen {
        struct_def: quote! {
          #[allow(unused)]
          pub struct #name {
            component: wick_packet::ComponentReference,
            inherent: flow_component::InherentContext
          }
        },
        struct_impl: quote! {
          #(#configs)*
          impl #name {
            pub fn new(component: wick_packet::ComponentReference, inherent: flow_component::InherentContext) -> Self {
              Self { component, inherent }
            }
            #[allow(unused)]
            pub fn component(&self) -> &wick_packet::ComponentReference {
              &self.component
            }
            #(#op_fns)*
          }
        },
        inputs: op_inputs,
        outputs: op_outputs,
      }
    })
    .collect_vec();

  let mod_name = Ident::new(name, Span::call_site());
  quote! {
    #[cfg(target_family = "wasm")]
    mod #mod_name {
      #[allow(unused)]
      use super::*;

      #(#components)*
    }
    #[cfg(target_family = "wasm")]
    pub use #mod_name::*;

  }
}

struct OperationCodegen {
  impls: TokenStream,
  input: Option<TokenStream>,
  output: Option<TokenStream>,
}

fn operation_impls(config: &mut Config, ops: &[OperationSignature]) -> Vec<OperationCodegen> {
  let dir = Direction::In;
  let raw = false;
  ops
    .iter()
    .map(|op| {
      config.add_dep(Dependency::WickComponent);

      let op_name = op.name();
      let name_raw = id(&snake(&format!("{}_raw", op_name)));
      let name = id(&snake(op_name));
      let (op_config_id, op_config_pair, set_context) = if !op.config().is_empty() {
        let id = id(&named_config_id(op.name()));
        (quote!{op_config,}, quote! { op_config: #id, }, quote! { Some(op_config.into()) })
      } else {
        (quote!{}, quote! {}, quote! { None })
      };
      let inputs = expand_input_fields(config, op.inputs(), dir, false);
      let encode_inputs = encoded_inputs(op.inputs());
      let merge_inputs = merged_inputs(op.inputs());
      let response_stream_types = expand_field_types(config, op.outputs(), dir, raw);
      let fan_out: Vec<_> = fields_to_tuples(config, op.outputs(), dir, raw);
      let types = f::maybe_parens(response_stream_types);

      let impls = quote! {
        #[allow(unused)]
        pub fn #name(&self, #op_config_pair #(#inputs),*) -> std::result::Result<#types,wick_packet::Error> {
          #(#encode_inputs)*
          let stream = wick_component::empty();
          let stream = #merge_inputs;
          let stream = wick_packet::PacketStream::new(Box::pin(stream));
          let mut stream = self.#name_raw(#op_config_id stream)?;
          Ok(wick_component::payload_fan_out!(stream, raw: false, wick_component::BoxError, [#(#fan_out),*]))
        }

        #[allow(unused)]
        pub fn #name_raw<T:Into<wick_packet::PacketStream>>(&self, #op_config_pair stream: T) -> std::result::Result<wick_packet::PacketStream,wick_packet::Error> {
          Ok(self.component.call(#op_name, stream.into(), #set_context, self.inherent.clone().into())?)
        }
      };
      let input = None;
      let output = None;
      OperationCodegen { impls, input, output }
    })
    .collect_vec()
}

fn named_configs(config: &mut Config, ops: &[OperationSignature]) -> Vec<TokenStream> {
  ops
    .iter()
    .filter_map(|op| {
      if op.config().is_empty() {
        None
      } else {
        let config_name = named_config_id(op.name());
        let config_id = id(&config_name);
        let def = op_config(config, &config_name, op);
        Some(quote! {
          #def

          impl From<#config_id> for wick_packet::RuntimeConfig {
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
        let #id = #id.map(wick_component::wick_packet::into_packet(#name));
      }
    })
    .collect_vec()
}

fn merged_inputs(fields: &[Field]) -> TokenStream {
  let start = id("stream");

  let tokens = fields.iter().fold(quote! {#start}, |acc: TokenStream, next| {
    let name = id(&snake(next.name()));
    quote! {
      #acc.merge(#name)
    }
  });
  let done_packets = fields.iter().map(|next| {
    let name = next.name();
    quote! {
      Ok(Packet::done(#name))
    }
  });

  quote! {
    #tokens.chain(wick_component::iter_raw(vec![#(#done_packets),*]))
  }
}
