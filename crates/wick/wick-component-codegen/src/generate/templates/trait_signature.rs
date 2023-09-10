use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::dependency::Dependency;
use crate::generate::expand_type::expand_type;
use crate::generate::ids::*;
use crate::generate::{config, f, Direction};

pub(crate) fn trait_signature(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let trait_name = id("Operation");
  let impl_name = id(&snake(op.name()));

  let inputs = op
    .inputs()
    .iter()
    .map(|i| {
      let port_name = id(&snake(&i.name));
      let port_type = expand_type(config, Direction::In, false, config.raw, &i.ty);
      quote! {#port_name: WickStream<#port_type>}
    })
    .collect_vec();

  let traits = f::gen_if(
    config.op_traits,
    || {
      config.add_dep(Dependency::AsyncTrait);
    },
    quote! {

      #[async_trait::async_trait(?Send)]
      #[cfg(target_family = "wasm")]
      pub trait #trait_name {
        type Error;
        type Outputs;
        type Config: std::fmt::Debug;

        #[allow(unused)]
        async fn #impl_name(#(#inputs,)* outputs: Self::Outputs, ctx: wick_component::flow_component::Context<Self::Config>) -> std::result::Result<(),Self::Error>;
      }

      #[async_trait::async_trait]
      #[cfg(not(target_family = "wasm"))]
      pub trait #trait_name {
        type Error: Send ;
        type Outputs: Send;
        type Config: std::fmt::Debug + Send ;

        #[allow(unused)]
        async fn #impl_name(#(#inputs,)* outputs: Self::Outputs, ctx: wick_component::flow_component::Context<Self::Config>) -> std::result::Result<(),Self::Error>;
      }
    },
  );

  quote! {
    #traits
  }
}
