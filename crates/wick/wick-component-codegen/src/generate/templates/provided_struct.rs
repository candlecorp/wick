use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::{Binding, InterfaceDefinition};

use crate::generate::ids::*;
use crate::*;

pub(crate) fn provided_struct(_config: &Config, required: &[Binding<InterfaceDefinition>]) -> TokenStream {
  let required_names = required
    .iter()
    .map(|r: &Binding<InterfaceDefinition>| {
      let name = id(&snake(r.id()));
      let orig_name = r.id();
      let response_name = id(&component_id(r));
      quote! { #name : #response_name::new(config.provided.get(#orig_name).cloned().unwrap(), inherent.clone()) }
    })
    .collect_vec();
  let fields = required
    .iter()
    .map(|v| {
      let name = id(&snake(v.id()));
      let uc_name = id(&component_id(v));
      quote! {pub #name: #uc_name}
    })
    .collect_vec();
  quote! {
    #[allow(unused)]
    #[cfg(target_family = "wasm")]
    mod provided_wasm {
      #[allow(unused)]
      use super::*;
      pub(crate) struct Provided {
        #(#fields),*
      }

      pub(crate) fn get_provided(inherent: wick_component::flow_component::InherentContext) -> Provided {
        let config = get_config();
        Provided {
          #(#required_names,)*
        }
      }

      pub(crate) trait ProvidedContext {
        fn provided(&self) -> Provided;
      }

      impl<T> ProvidedContext for wick_component::flow_component::Context<T> where T:std::fmt::Debug{
        fn provided(&self) -> Provided {
          get_provided(self.inherent.clone())
        }
      }
    }
    #[cfg(target_family = "wasm")]
    pub(crate) use provided_wasm::*;
  }
}
