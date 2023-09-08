use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::Binding;

use crate::generate::ids::*;

pub(crate) fn imported_component_container<T>(name: &str, required: &[Binding<T>]) -> TokenStream {
  let mod_id = id(&format!("{}_wasm", snake(name)));
  let method_id = id(&snake(name));
  let struct_id = id(&pascal(name));
  let trait_id = id(&format!("{}Context", pascal(name)));

pub(crate) fn imported_component_container<T>(name: &str, required: &[Binding<T>]) -> TokenStream {
  let mod_id = id(&format!("{}_wasm", snake(name)));
  let method_id = id(&snake(name));
  let struct_id = id(&pascal(name));
  let trait_id = id(&format!("{}Context", pascal(name)));

  let required_names = required
    .iter()
    .map(|r: &Binding<T>| {
      let name = id(&snake(r.id()));
      let orig_name = r.id();
      let response_name = id(&component_id(r));
      quote! { #name : #response_name::new(config.#method_id.get(#orig_name).cloned().unwrap(), inherent.clone()) }
      quote! { #name : #response_name::new(config.#method_id.get(#orig_name).cloned().unwrap(), inherent.clone()) }
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
    mod #mod_id {
    mod #mod_id {
      #[allow(unused)]
      use super::*;
      pub(crate) struct #struct_id {
      pub(crate) struct #struct_id {
        #(#fields),*
      }

      pub(crate) trait #trait_id {
        fn #method_id(&self) -> #struct_id;
      pub(crate) trait #trait_id {
        fn #method_id(&self) -> #struct_id;
      }

      impl<T> #trait_id for wick_component::flow_component::Context<T> where T:std::fmt::Debug{
        fn #method_id(&self) -> #struct_id {
          let config = get_config();
          let inherent = self.inherent.clone();
          #struct_id {
            #(#required_names),*
          }
        }
      }
    }
    #[cfg(target_family = "wasm")]
    pub(crate) use #mod_id::*;
    pub(crate) use #mod_id::*;
  }
}
