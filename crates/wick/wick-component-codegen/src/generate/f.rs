#![allow(unused)]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde_json::Value;
use wick_config::config::components::OperationConfig;
use wick_config::config::OperationDefinition;
use wick_interface_types::{Field, Type};

use super::config;
use crate::generate::ids::{id, snake};
use crate::generate::{expand_type, Direction};

fn is_defaultable(ty: &Type) -> bool {
  matches!(
    ty,
    Type::List { .. } | Type::Optional { .. } | Type::Map { .. } | Type::Object
  )
}

pub(crate) fn field_pair(
  config: &mut config::Config,
  imported: bool,
  serde: bool,
) -> impl FnMut(&Field) -> TokenStream + '_ {
  move |field: &Field| {
    let name = &field.name;
    let id = id(&snake(name));
    let ty = expand_type(config, Direction::In, imported, &field.ty);
    let desc = field
      .description
      .as_ref()
      .map_or_else(|| quote! {}, |desc| quote! {#[doc = #desc]});

    let serde = if serde {
      let default = (!field.required || is_defaultable(field.ty())).then(|| quote! {#[serde(default)]});
      let skip_if = match field.ty() {
        wick_interface_types::Type::List { .. } => quote! { #[serde(skip_serializing_if = "Vec::is_empty")] },
        wick_interface_types::Type::Optional { .. } => quote! { #[serde(skip_serializing_if = "Option::is_none")] },
        wick_interface_types::Type::Map { .. } => {
          quote! { #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")] }
        }
        _ => quote! {},
      };

      #[allow(clippy::match_single_binding)]
      let deserialize_with = match field.ty() {
        wick_interface_types::Type::Datetime => quote! {
          #[serde(deserialize_with = "wick_component::datetime::serde::from_str_or_integer")]
        },
        _ => quote! {},
      };

      quote! {
        #[serde(rename = #name)]
        #default
        #deserialize_with
        #skip_if
      }
    } else {
      quote! {}
    };
    quote! {
      #desc
      #serde
      pub #id: #ty
    }
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

fn op_names(ops: &[OperationDefinition]) -> Vec<String> {
  ops.iter().map(|op| op.name().to_owned()).collect()
}

fn field_names(fields: &[Field]) -> Vec<String> {
  fields.iter().map(|field| field.name().to_owned()).collect()
}
