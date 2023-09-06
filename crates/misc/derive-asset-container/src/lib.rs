//! This crate provides a derive macro for the AssetManager trait.
//!
//!
//! # Example
//!
//! ```rust
//! use derive_asset_container::AssetManager;
//!
//! #[derive(Clone, AssetManager)]
//! #[asset(asset(TestAsset))]
//! struct Struct {
//!   field: TestAsset,
//!   inner: InnerStruct,
//! }
//!
//! #[derive(Clone, AssetManager)]
//! #[asset(asset(TestAsset), lazy)]
//! struct InnerStruct {
//!   field: TestAsset,
//! }
//! ```

// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::await_holding_lock,
  clippy::borrow_as_ptr,
  clippy::branches_sharing_code,
  clippy::cast_lossless,
  clippy::clippy::collection_is_never_read,
  clippy::cloned_instead_of_copied,
  clippy::cognitive_complexity,
  clippy::create_dir,
  clippy::deref_by_slicing,
  clippy::derivable_impls,
  clippy::derive_partial_eq_without_eq,
  clippy::equatable_if_let,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::expl_impl_clone_on_copy,
  clippy::explicit_deref_methods,
  clippy::explicit_into_iter_loop,
  clippy::explicit_iter_loop,
  clippy::filetype_is_file,
  clippy::flat_map_option,
  clippy::format_push_string,
  clippy::fn_params_excessive_bools,
  clippy::future_not_send,
  clippy::get_unwrap,
  clippy::implicit_clone,
  clippy::if_then_some_else_none,
  clippy::impl_trait_in_params,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::inherent_to_string,
  clippy::iter_not_returning_iterator,
  clippy::large_types_passed_by_value,
  clippy::large_include_file,
  clippy::let_and_return,
  clippy::manual_assert,
  clippy::manual_ok_or,
  clippy::manual_split_once,
  clippy::manual_let_else,
  clippy::manual_string_new,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::missing_enforced_import_renames,
  clippy::missing_assert_message,
  clippy::missing_const_for_fn,
  clippy::must_use_candidate,
  clippy::mut_mut,
  clippy::needless_for_each,
  clippy::needless_option_as_deref,
  clippy::needless_pass_by_value,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::non_send_fields_in_send_ty,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::option_option,
  clippy::rc_mutex,
  clippy::redundant_else,
  clippy::same_name_method,
  clippy::semicolon_if_nothing_returned,
  clippy::str_to_string,
  clippy::string_to_string,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::trivial_regex,
  clippy::try_err,
  clippy::unnested_or_patterns,
  clippy::unused_async,
  clippy::unwrap_or_else_default,
  clippy::useless_let_if_seq,
  bad_style,
  clashing_extern_declarations,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![warn(clippy::exhaustive_enums)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow(clippy::expect_used)]

use asset_container::AssetFlags;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use structmeta::{NameArgs, StructMeta};
use syn::{parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Type};

#[derive(Debug, StructMeta)]
struct TypeOpts {
  lazy: bool,
  #[struct_meta(name = "asset")]
  ty: NameArgs<Type>,
}

/// The derive macro for the AssetManager trait.
#[proc_macro_derive(AssetManager, attributes(asset_managers, asset))]
pub fn derive_asset_container(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree.
  let ast = parse_macro_input!(input as DeriveInput);

  // Extract the name of the struct we're deriving the trait for.
  let name = &ast.ident;

  // Parse the attribute arguments.
  let opts = ast
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("asset"))
    .map(|attribute| attribute.parse_args::<TypeOpts>().expect("invalid attribute arguments"))
    .expect("no asset attribute");

  match ast.data {
    syn::Data::Struct(ref data) => impl_struct(name, data, opts),
    syn::Data::Enum(ref data) => impl_enum(name, data, opts),
    _ => panic!("Only structs and enums can derive the Assets trait."),
  }
}

fn has_skip(attr: &[Attribute]) -> bool {
  attr
    .iter()
    .find(|attr| attr.path().is_ident("asset"))
    .map_or(false, |attr| {
      let ident = attr.parse_args::<Ident>().expect("invalid attribute arguments");
      ident == "skip"
    })
}

fn impl_struct(name: &Ident, data: &DataStruct, opts: TypeOpts) -> TokenStream {
  let fields = &data.fields;
  // Generate a list of field names as strings.
  let asset_fields = fields
    .iter()
    .filter(|field| field.ty == opts.ty.args)
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| {
      field
        .ident
        .as_ref()
        .map_or_else(|| panic!("Unnamed fields are not supported."), |ident| ident.clone())
    })
    .collect::<Vec<Ident>>();

  // Generate a list of field names as strings.
  let inner_managers = fields
    .iter()
    .filter(|field| field.ty != opts.ty.args)
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| {
      field
        .ident
        .as_ref()
        .map_or_else(|| panic!("Unnamed fields are not supported."), |ident| ident.clone())
    })
    .collect::<Vec<Ident>>();
  let flags = if opts.lazy {
    AssetFlags::Lazy
  } else {
    AssetFlags::empty()
  }
  .bits();
  let flags = quote! {#flags};
  let asset_type = opts.ty.args;

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &std::path::Path) {
            use asset_container::Asset;
            #(self.#asset_fields.update_baseurl(baseurl);)*
            #(self.#inner_managers.set_baseurl(baseurl);)*
          }

          fn assets(&self) -> asset_container::Assets<#asset_type> {
            let mut assets = asset_container::Assets::new(vec![],self.get_asset_flags());
            #(assets.push(&self.#asset_fields);)*
            #(assets.extend(self.#inner_managers.assets());)*
            assets
          }

          fn get_asset_flags(&self) -> u32 {
            #flags
          }
      }
  };
  TokenStream::from(output)
}

fn impl_enum(name: &Ident, data: &DataEnum, opts: TypeOpts) -> TokenStream {
  let variants = &data.variants;

  let asset_variants = variants
    .iter()
    .filter(|v| v.fields.iter().any(|f| f.ty == opts.ty.args))
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| field.ident.clone())
    .collect::<Vec<Ident>>();

  let inner_managers = variants
    .iter()
    .filter(|v| v.fields.iter().any(|f| f.ty != opts.ty.args))
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| field.ident.clone())
    .collect::<Vec<Ident>>();

  let flags = if opts.lazy {
    AssetFlags::Lazy
  } else {
    AssetFlags::empty()
  }
  .bits();
  let flags = quote! {#flags};
  let asset_type = opts.ty.args;

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &std::path::Path) {
            use asset_container::Asset;
            match self {
              #(Self::#asset_variants(v) => {
                v.update_baseurl(baseurl);
              })*
              #(Self::#inner_managers(v) => {
                v.set_baseurl(baseurl);
              })*
              _ => {}
            }
          }

          fn assets(&self) -> asset_container::Assets<#asset_type> {
            let mut assets = asset_container::Assets::new(vec![],self.get_asset_flags());
            match self {
              #(Self::#asset_variants(v) => {
                assets.push(v);
              })*
              #(Self::#inner_managers(v) => {
                assets.extend(asset_container::AssetManager::assets(v));
              })*
              _ => {}
            }
            assets
          }

          fn get_asset_flags(&self) -> u32 {
            #flags
          }
      }
  };
  TokenStream::from(output)
}
