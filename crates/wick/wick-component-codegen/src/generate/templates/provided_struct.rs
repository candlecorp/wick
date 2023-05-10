use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_config::config::BoundInterface;

use crate::generate::ids::*;
use crate::*;

pub(crate) fn provided_struct(_config: &Config, required: &[BoundInterface]) -> TokenStream {
  let required_names = required
    .iter()
    .map(|r: &BoundInterface| {
      let name = id(&snake(r.id()));
      let orig_name = r.id();
      let response_name = id(&component_id(r));
      quote! { #name : #response_name::new(config.provided.get(#orig_name).cloned().unwrap()) }
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
    pub(crate) struct Provided {
      #(#fields),*
    }

    pub(crate) fn get_provided() -> Provided {
      let config = get_config();
      Provided {
        #(#required_names,)*
      }

    }
  }
}
