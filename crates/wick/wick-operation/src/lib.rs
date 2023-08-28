// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::unused_async,
  clippy::missing_enforced_import_renames,
  clippy::nonstandard_macro_braces,
  clippy::rc_mutex,
  clippy::unwrap_or_else_default,
  clippy::manual_split_once,
  clippy::derivable_impls,
  clippy::needless_option_as_deref,
  clippy::iter_not_returning_iterator,
  clippy::same_name_method,
  clippy::manual_assert,
  clippy::non_send_fields_in_send_ty,
  clippy::equatable_if_let,
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
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow()]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::crate_name;
use quote::{quote, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::token::PathSep;
use syn::{parse_macro_input, FnArg, ItemFn, Meta, PathSegment, ReturnType, Token};

enum Adapter {
  BinaryInterleavedPairs,
  BinaryPairedRightStream,
  UnarySimple,
}

impl Adapter {
  fn from_segments(s: &Punctuated<PathSegment, PathSep>) -> Option<Self> {
    let mut s = s.iter().rev();
    let last = s.next()?.ident.to_string();
    match last.as_str() {
      "binary_interleaved_pairs" => Some(Adapter::BinaryInterleavedPairs),
      "binary_paired_right_stream" => Some(Adapter::BinaryPairedRightStream),
      "unary_simple" => Some(Adapter::UnarySimple),
      _ => None,
    }
  }

  fn available_adapters() -> Vec<Self> {
    vec![
      Adapter::BinaryInterleavedPairs,
      Adapter::BinaryPairedRightStream,
      Adapter::UnarySimple,
    ]
  }
}

impl std::fmt::Display for Adapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Adapter::BinaryInterleavedPairs => write!(f, "binary_interleaved_pairs"),
      Adapter::BinaryPairedRightStream => write!(f, "binary_paired_right_stream"),
      Adapter::UnarySimple => write!(f, "unary_simple"),
    }
  }
}

impl ToTokens for Adapter {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    Ident::new(&self.to_string(), Span::call_site()).to_tokens(tokens)
  }
}

#[proc_macro_attribute]
pub fn operation(attr: TokenStream, item: TokenStream) -> TokenStream {
  let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
  let args = match parser.parse2(attr.into()) {
    Ok(args) => args,
    Err(e) => panic!("{}", e),
  };

  let input = parse_macro_input!(item as ItemFn);

  let args = args.into_iter().collect::<Vec<_>>();

  let adapter = match args.as_slice() {
    [Meta::Path(path)] => Adapter::from_segments(&path.segments),
    _ => {
      panic!(
        "unsupported attributes supplied: {}, available adapters are {}",
        quote! { args },
        Adapter::available_adapters()
          .iter()
          .map(|a| a.to_string())
          .collect::<Vec<_>>()
          .join(", ")
      );
    }
  }
  .unwrap_or_else(|| {
    panic!(
      "unsupported attributes supplied: {}, available adapters are {}",
      quote! { args },
      Adapter::available_adapters()
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ")
    )
  });
  let _ = crate_name("wick-component").expect("wick-component needs to be added in `Cargo.toml`");

  expand_wrapper(adapter, &input)
}

/// Emit code for a wrapper function around a test function.
fn expand_wrapper(adapter: Adapter, wrappee: &ItemFn) -> TokenStream {
  let attrs = &wrappee.attrs;
  let async_ = &wrappee.sig.asyncness;
  let fn_body = &wrappee.block;
  let fn_name = &wrappee.sig.ident;
  let fn_arg_types_outer = &wrappee
    .sig
    .inputs
    .iter()
    .cloned()
    .map(|mut arg| {
      if let FnArg::Typed(pat) = &mut arg {
        #[allow(clippy::single_match)]
        match pat.pat.as_mut() {
          syn::Pat::Ident(id) => {
            id.mutability = None;
          }
          _ => {}
        }
      }
      arg
    })
    .collect::<Vec<_>>();
  let fn_arg_types = &wrappee.sig.inputs.iter().collect::<Vec<_>>();
  let fn_arg_names = &wrappee
    .sig
    .inputs
    .iter()
    .filter_map(|arg| {
      if let FnArg::Typed(pat) = arg {
        match pat.pat.as_ref() {
          syn::Pat::Ident(id) => Some(&id.ident),
          _ => None,
        }
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  let fn_return = match &wrappee.sig.output {
    ReturnType::Default => quote! {()},
    ReturnType::Type(_, type_) => quote! {#type_},
  };
  let fn_wrapper_name = Ident::new(&format!("real_{}", fn_name), Span::call_site());

  let (async_, await_) = if async_.is_some() {
    (Some(quote! {async}), Some(quote! {.await}))
  } else {
    (None, None)
  };

  let result = quote! {
    wick_component::#adapter !(#fn_name);

    #(#attrs)*
    fn #fn_name(#(#fn_arg_types_outer),*) -> std::pin::Pin<Box<dyn std::future::Future<Output = #fn_return> + 'static>> {
      Box::pin(async move { #fn_wrapper_name(#(#fn_arg_names),*)#await_ })
    }

    #async_ fn #fn_wrapper_name(#(#fn_arg_types),*) -> #fn_return {
      #fn_body
    }

  };

  result.into()
}
