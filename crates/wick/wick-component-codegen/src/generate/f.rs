#![allow(unused)]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde_json::Value;
use wick_config::config::OperationSignature;
use wick_interface_types::Field;

use super::config;
use crate::generate::ids::{id, snake};
use crate::generate::{expand_type, Direction};

pub(crate) fn field_pair(config: &mut config::Config, imported: bool) -> impl FnMut(&Field) -> TokenStream + '_ {
  move |f| {
    let name = id(&snake(&f.name));
    let ty = expand_type(config, Direction::In, imported, &f.ty);
    quote! {pub #name: #ty}
  }
}

pub(crate) fn field_default(config: &mut config::Config, imported: bool) -> impl FnMut(&Field) -> TokenStream + '_ {
  move |f| {
    let name = id(&snake(&f.name));
    let default = default_val()(f.default());
    quote! {#name: #default}
  }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn gen_if(condition: bool, mut func: impl FnMut(), value: TokenStream) -> TokenStream {
  if condition {
    func();
    quote! { #value }
  } else {
    quote! {}
  }
}

pub(crate) fn default_val() -> impl FnMut(Option<&Value>) -> TokenStream {
  move |f| f.map_or_else(|| quote! {Default::default()}, default_from_val)
}

fn from_json(value: &Value) -> TokenStream {
  let json_str = serde_json::to_string(&value).unwrap();
  quote! {serde_json::from_str(#json_str).unwrap()}
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn maybe_parens(list: Vec<impl ToTokens>) -> TokenStream {
  if list.len() == 1 {
    quote! { #(#list),* }
  } else {
    quote! { (#(#list),*) }
  }
}

fn default_from_val(value: &Value) -> TokenStream {
  match value {
    Value::Bool(b) => quote! {#b},
    Value::Number(n) => {
      if n.is_f64() {
        let n = n.as_f64().unwrap();
        quote! {#n}
      } else {
        let n = n.as_i64().unwrap();
        quote! {#n}
      }
    }
    Value::String(s) => quote! {#s},
    Value::Array(_) => from_json(value),
    Value::Object(_) => from_json(value),
    Value::Null => quote! {None},
  }
}

fn op_names(ops: &[OperationSignature]) -> Vec<String> {
  ops.iter().map(|op| op.name().to_owned()).collect()
}

fn field_names(fields: &[Field]) -> Vec<String> {
  fields.iter().map(|field| field.name().to_owned()).collect()
}
