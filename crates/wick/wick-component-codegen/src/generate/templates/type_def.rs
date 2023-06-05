#![allow(clippy::same_name_method)]
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::{EnumDefinition, StructDefinition, TypeDefinition};

use crate::generate::ids::*;
use crate::generate::{config, f};

bitflags::bitflags! {
  #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub(crate) struct TypeOptions: u32 {
      const Defaults = 0b00000001;
  }
}

pub(crate) fn type_def<'a>(
  config: &mut config::Config,
  ty: &'a TypeDefinition,
  options: TypeOptions,
) -> (Vec<&'a str>, TokenStream) {
  match ty {
    TypeDefinition::Enum(ty) => gen_enum(config, ty, options),
    TypeDefinition::Struct(ty) => gen_struct(config, ty, options),
  }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn gen_enum<'a>(
  _config: &config::Config,
  ty: &'a EnumDefinition,
  _options: TypeOptions,
) -> (Vec<&'a str>, TokenStream) {
  let (path_parts, item_part) = get_typename_parts(&ty.name);
  let name = id(item_part);
  let variants = ty
    .variants
    .iter()
    .map(|v| {
      let name = id(&enumvariant_name(v));
      let description = v
        .description
        .as_ref()
        .map_or_else(|| quote! {}, |desc| quote! {#[doc = #desc]});

      quote! {
        #description
        #name
      }
    })
    .collect_vec();

  let from_index_arms = ty
    .variants
    .iter()
    .filter_map(|v| {
      let identname = id(&enumvariant_name(v));
      v.index.map(|_| quote! {i => Self::#identname})
    })
    .collect_vec();

  let display_match_arms = ty
    .variants
    .iter()
    .map(|v| {
      let identname = id(&enumvariant_name(v));
      let name = v.name.clone();
      quote! {Self::#identname => f.write_str(#name)}
    })
    .collect_vec();
  let value_match_arms = ty
    .variants
    .iter()
    .map(|v| {
      let identname = id(&enumvariant_name(v));
      let name = v
        .value
        .as_ref()
        .map_or_else(|| quote! {None}, |value| quote! { Some(#value) });
      quote! {Self::#identname => #name}
    })
    .collect_vec();

  let fromstr_match_arms = ty
    .variants
    .iter()
    .filter_map(|v| {
      let identname = id(&enumvariant_name(v));
      v.value.as_ref().map(|name| quote! {#name => Ok(Self::#identname)})
    })
    .collect_vec();

  let description = ty
    .description
    .as_ref()
    .map_or_else(|| quote! {}, |desc| quote! {#[doc = #desc]});

  let enum_impl = quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    #description
    pub enum #name {
      #(#variants,)*

    }

    impl #name {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
      pub fn value(&self) -> Option<&'static str> {
        #[allow(clippy::match_single_binding)]
        match self {
          #(#value_match_arms,)*
        }
      }
    }

    impl TryFrom<u32> for #name {
      type Error = u32;
      fn try_from(i: u32) -> Result<Self, Self::Error> {
        #[allow(clippy::match_single_binding)]
        match i {
          #(#from_index_arms,)*
          _ => Err(i)
        }
      }
    }

    impl std::str::FromStr for #name {
      type Err = String;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::match_single_binding)]
        match s {
          #(#fromstr_match_arms,)*
          _ => Err(s.to_owned())
        }
      }
    }

    impl std::fmt::Display for #name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          #(#display_match_arms,)*
        }
      }
    }
  };
  (path_parts, enum_impl)
}

pub(crate) fn gen_struct<'a>(
  config: &mut config::Config,
  ty: &'a StructDefinition,
  options: TypeOptions,
) -> (Vec<&'a str>, TokenStream) {
  let (module_parts, item_part) = get_typename_parts(&ty.name);
  let imported = ty.imported;

  let name = id(item_part);

  let fields = ty
    .fields
    .iter()
    .map(f::field_pair(config, imported, true))
    .collect_vec();

  let (derive, default_impl) = if ty.fields.is_empty() {
    (
      quote! {
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
      },
      quote! {},
    )
  } else {
    let fields = ty.fields.iter().map(f::field_default(config, imported)).collect_vec();
    (
      quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
      },
      quote! {
        impl Default for #name {
          fn default() -> Self {
            Self {
              #(#fields,)*
            }
          }
        }
      },
    )
  };

  let default_impl = f::gen_if(options == TypeOptions::Defaults, || {}, default_impl);

  let description = ty
    .description
    .as_ref()
    .map_or_else(|| quote! {}, |desc| quote! {#[doc = #desc]});

  let item = quote! {
    #derive
    #description
    pub struct #name {
      #(#fields,)*
    }
    #default_impl
  };
  (module_parts, item)
}
