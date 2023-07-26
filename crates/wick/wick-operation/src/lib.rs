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
