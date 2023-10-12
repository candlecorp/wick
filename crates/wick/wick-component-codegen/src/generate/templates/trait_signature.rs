use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::OperationSignature;

use crate::generate::config;
use crate::generate::dependency::Dependency;
use crate::generate::ids::*;

pub(crate) fn trait_signature(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let trait_name = id("Operation");
  let impl_name = id(&snake(op.name()));

  let traits = config.op_traits.then(||{
    config.add_dep(Dependency::AsyncTrait);
    quote! {

      #[async_trait::async_trait(?Send)]
      #[cfg(target_family = "wasm")]
      pub trait #trait_name {
        type Error;
        type Inputs;
        type Outputs;
        type Config: std::fmt::Debug;

        #[allow(unused)]
        async fn #impl_name(inputs: Self::Inputs, outputs: Self::Outputs, ctx: wick_component::flow_component::Context<Self::Config>) -> std::result::Result<(),Self::Error>;
      }

      #[async_trait::async_trait]
      #[cfg(not(target_family = "wasm"))]
      pub trait #trait_name {
        type Error: Send ;
        type Inputs: Send;
        type Outputs: Send;
        type Config: std::fmt::Debug + Send ;

        #[allow(unused)]
        async fn #impl_name(inputs: Self::Inputs, outputs: Self::Outputs, ctx: wick_component::flow_component::Context<Self::Config>) -> std::result::Result<(),Self::Error>;
      }
    }
  });

  quote! {
    #traits
  }
}
