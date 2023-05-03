use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::BoundInterface;

use crate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{f, Direction};
use crate::*;

pub(crate) fn response_streams(config: &mut Config, required: Vec<BoundInterface>) -> TokenStream {
  let fields = required
    .into_iter()
    .map(|v| {
      let name = id(&format!("{}Component", &pascal(&v.id)));
      let ops = v
        .kind
        .operations()
        .values()
        .map(|op| {
          let op_name = op.name();
          let name = id(&snake(op_name));
          let response_streams: Vec<_> = op
            .outputs()
            .iter()
            .map(|output| {
              (
                output.name.clone(),
                expand_type(config, Direction::In, false, &output.ty),
              )
            })
            .collect_vec();
          let response_stream_types = response_streams.iter().map(|(_, ty)| quote! { WickStream<#ty>});
          let fan_out: Vec<_> = response_streams
            .iter()
            .map(|(n, t)| {
              quote! {
                (#n, #t)
              }
            })
            .collect();
          let types = f::sanitize_list(response_stream_types.collect());
          config.add_dep(Dependency::WickComponent);
          quote! {
            pub fn #name(&self, input: wick_packet::PacketStream) -> std::result::Result<#types,wick_packet::Error> {
              let mut stream = self.component.call(#op_name, input)?;
              Ok(wick_component::payload_fan_out!(stream, raw: false, [#(#fan_out),*]))
            }
          }
        })
        .collect_vec();

      config.add_dep(Dependency::WickPacket);
      quote! {
        pub struct #name {
          component: wick_packet::ComponentReference,
        }

        impl #name {
          pub fn new(component: wick_packet::ComponentReference) -> Self {
            Self { component }
          }
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
