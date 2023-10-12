use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::dependency::Dependency;
use super::{config, Direction};
use crate::generate::ids::get_typename_parts;

pub(super) fn expand_type(
  config: &mut config::Config,
  dir: Direction,
  imported: bool,
  raw: bool,
  ty: &wick_interface_types::Type,
) -> TokenStream {
  if raw && dir != Direction::Out {
    return quote! { wick_component::wick_packet::Packet };
  }
  match ty {
    wick_interface_types::Type::Bool => quote! { bool },
    wick_interface_types::Type::U8 => quote! { u8 },
    wick_interface_types::Type::U16 => quote! { u16 },
    wick_interface_types::Type::U32 => quote! { u32 },
    wick_interface_types::Type::U64 => quote! { u64 },
    wick_interface_types::Type::I8 => quote! { i8 },
    wick_interface_types::Type::I16 => quote! { i16 },
    wick_interface_types::Type::I32 => quote! { i32 },
    wick_interface_types::Type::I64 => quote! { i64 },
    wick_interface_types::Type::F32 => quote! { f32 },
    wick_interface_types::Type::F64 => quote! { f64 },
    wick_interface_types::Type::String => quote! { String },
    wick_interface_types::Type::List { ty } => {
      let ty = expand_type(config, dir, imported, raw, ty);
      quote! { Vec<#ty> }
    }
    wick_interface_types::Type::Bytes => {
      config.add_dep(Dependency::Bytes);
      quote! {wick_component::Bytes}
    }
    wick_interface_types::Type::Named(name) => {
      let (mod_parts, item_part) = get_typename_parts(name);
      let mod_parts = mod_parts.iter().map(|p| Ident::new(p, Span::call_site()));
      let ty = Ident::new(item_part, Span::call_site());
      let location = if imported {
        quote! {}
      } else {
        quote! {types::}
      };
      quote! {#location #(#mod_parts ::)*#ty}
    }
    wick_interface_types::Type::Optional { ty } => {
      let ty = expand_type(config, dir, imported, raw, ty);
      quote! { Option<#ty> }
    }
    wick_interface_types::Type::Map { key, value } => {
      let key = expand_type(config, dir, imported, raw, key);
      let value = expand_type(config, dir, imported, raw, value);
      quote! { std::collections::HashMap<#key,#value> }
    }
    #[allow(deprecated)]
    wick_interface_types::Type::Link { .. } => {
      config.add_dep(Dependency::WickComponent);
      quote! {wick_component::wick_packet::ComponentReference}
    }
    wick_interface_types::Type::Datetime => {
      config.add_dep(Dependency::Chrono);
      quote! {wick_component::datetime::DateTime}
    }
    wick_interface_types::Type::Object => {
      config.add_dep(Dependency::SerdeJson);
      quote! { wick_component::Value }
    }
    wick_interface_types::Type::AnonymousStruct(_) => todo!("implement anonymous struct in new codegen"),
  }
}
