use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use wick_config::config::Binding;
use wick_interface_types::{OperationSignature, OperationSignatures};

use super::{op_incoming, op_outgoing, op_simple_outgoing};
use crate::generate::dependency::Dependency;
use crate::generate::ids::*;
use crate::generate::templates::op_config;
use crate::*;

struct ComponentCodegen {
  mod_name: Ident,
  struct_def: TokenStream,
  struct_impl: TokenStream,
  op_modules: Vec<TokenStream>,
}

impl ToTokens for ComponentCodegen {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let mod_name = &self.mod_name;
    let mut mod_tokens = quote! {};
    for opt in &self.op_modules {
      mod_tokens.extend(opt.clone());
    }
    tokens.extend(quote! {
      pub(crate) mod #mod_name {
        use super::*;
        #mod_tokens
      }
    });

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
      let mod_name = id(&format!("{}_component", &snake(v.id())));
      let ops = operation_impls(config, &mod_name, &v.kind().operation_signatures());
      let (mut op_fns, mut op_modules) = (Vec::new(), Vec::new());
      for op in ops {
        op_fns.push(op.impls);
        op_modules.push(op.op_module);
      }

      config.add_dep(Dependency::WickPacket);
      ComponentCodegen {
        mod_name,
        struct_def: quote! {
          #[allow(unused)]
          pub struct #name {
            component: wick_packet::ComponentReference,
            inherent: flow_component::InherentContext
          }
        },
        struct_impl: quote! {
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
        op_modules,
      }
    })
    .collect_vec();

  let mod_name = Ident::new(name, Span::call_site());
  quote! {
    #[cfg(target_family = "wasm")]
    pub(crate) mod #mod_name {
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
  op_module: TokenStream,
}

fn operation_impls(config: &mut Config, mod_name: &Ident, ops: &[OperationSignature]) -> Vec<OperationCodegen> {
  ops
    .iter()
    .map(|op| {
      config.add_dep(Dependency::WickComponent);

      let op_name = op.name();
      let name_packets = id(&snake(&format!("{}_packets", op_name)));
      let name_raw = id(&snake(&format!("{}_raw", op_name)));
      let name = id(&snake(op_name));

      let impls = quote! {
        #[allow(unused)]
        pub fn #name(&self, op_config: #mod_name::#name::Config, mut inputs: impl Into<#mod_name::#name::Inputs>) -> std::result::Result<#mod_name::#name::Outputs,wick_packet::Error> {
          let mut stream = self.#name_raw(op_config, inputs.into().channel.take_rx().unwrap().boxed())?;
          let (_,outputs) = #mod_name::#name::process_incoming(stream);
          Ok(outputs)
        }

        #[allow(unused)]
        pub fn #name_packets(&self, op_config: #mod_name::#name::Config, stream: wick_packet::PacketStream) -> std::result::Result<wick_packet::PacketStream,wick_packet::Error> {
          Ok(wick_packet::from_wasmrs(self.#name_raw(op_config,wick_packet::packetstream_to_wasmrs(0,stream))?))
        }

        #[allow(unused)]
        pub fn #name_raw(&self, op_config: #mod_name::#name::Config, stream: wick_component::wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>) -> std::result::Result<wick_component::wasmrs_rx::BoxFlux<wick_component::wasmrs::Payload,wick_component::wasmrs::PayloadError>,wick_packet::Error> {
          Ok(self.component.call(#op_name, stream, Some(op_config.into()), self.inherent.clone().into())?)
        }
      };
      let input = op_outgoing(config, "Inputs",  op.inputs());
      let input_simple = op_simple_outgoing(config, "Request", "Inputs", op.inputs());
      let output = op_incoming(config, "Outputs",  op.outputs());
      let config = op_config(config, "Config", op);
      let op_module = quote!{
        pub(crate) mod #name {
          use super::*;
          #input
          #input_simple
          #output
          #config
        }
      };

      OperationCodegen { impls, op_module }
    })
    .collect_vec()
}
