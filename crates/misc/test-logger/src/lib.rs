// Copyright (C) 2019-2021 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as Tokens};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, ItemFn, Meta, ReturnType, Token};

/// A procedural macro for the `test` attribute.
///
/// The attribute can be used to define a test that has the `env_logger`
/// and/or `tracing` crates initialized (depending on the features used).
///
/// # Example
///
/// Specify the attribute on a per-test basis:
/// ```rust
/// # // doctests seemingly run in a slightly different environment where
/// # // `super`, which is what our macro makes use of, is not available.
/// # // By having a fake module here we work around that problem.
/// # #[cfg(feature = "log")]
/// # mod fordoctest {
/// # use logging::info;
/// # // Note that no test would actually run, regardless of `no_run`,
/// # // because we do not invoke the function.
/// #[test_logger::test]
/// fn it_works() {
///   info!("Checking whether it still works...");
///   assert_eq!(2 + 2, 4);
///   info!("Looks good!");
/// }
/// # }
/// ```
///
/// It can be very convenient to convert over all tests by overriding
/// the `#[test]` attribute on a per-module basis:
/// ```rust,no_run
/// # mod fordoctest {
/// use test_logger::test;
///
/// #[test]
/// fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
///
/// You can also wrap another attribute. For example, suppose you use
/// [`#[tokio::test]`](https://docs.rs/tokio/1.4.0/tokio/attr.test.html)
/// to run async tests:
/// ```
/// # mod fordoctest {
/// use test_logger::test;
///
/// #[test(tokio::test)]
/// async fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
  let args = match parser.parse2(attr.into()) {
    Ok(args) => args,
    Err(e) => panic!("{}", e),
  };

  let input = parse_macro_input!(item as ItemFn);

  let args = args.into_iter().collect::<Vec<_>>();

  let inner_test = match args.as_slice() {
    [] => parse_quote! { ::core::prelude::v1::test },
    [Meta::Path(path)] => quote! {#path},
    [Meta::List(list)] => {
      let path = &list.path;
      let args = &list.tokens;
      quote! { #path(#args) }
    }
    _ => {
      panic!("unsupported attributes supplied: {}", quote! { args })
    }
  };

  expand_wrapper(inner_test, &input)
}

/// Expand the initialization code for the `log` crate.
fn expand_logging_init() -> Tokens {
  let found_crate = crate_name("wick-logger").expect("wick-logger needs to be added in `Cargo.toml`");

  match found_crate {
    FoundCrate::Itself => quote! {
      let logging_options = crate::LoggingOptionsBuilder::default()
        .app_name("test")
        .otlp_endpoint(std::env::var("OTLP_ENDPOINT").ok())
        .levels(crate::LogFilters::with_level(crate::LogLevel::Trace))
        .build()
        .unwrap();
      let __guard = crate::init_test(&logging_options);
    },
    FoundCrate::Name(name) => {
      let ident = Ident::new(&name, Span::call_site());

      quote! {
        let logging_options = #ident::LoggingOptionsBuilder::default()
        .app_name("test")
        .otlp_endpoint(std::env::var("OTLP_ENDPOINT").ok())
        .levels(#ident::LogFilters::with_level(#ident::LogLevel::Trace))
        .build()
        .unwrap();
        let __guard = #ident::init_test(&logging_options);
      }
    }
  }
}

/// Emit code for a wrapper function around a test function.
fn expand_wrapper(inner_test: Tokens, wrappee: &ItemFn) -> TokenStream {
  let attrs = &wrappee.attrs;
  let async_ = &wrappee.sig.asyncness;
  let await_ = if async_.is_some() {
    quote! {.instrument(span).await}
  } else {
    quote! {}
  };
  let enter_ = if async_.is_some() {
    quote! {use tracing::Instrument;}
  } else {
    quote! {let _guard = span.enter();}
  };
  let exit_ = if async_.is_some() {
    quote! {
      // TODO Find a better way to ensure the opentel exporter has flushed.
      tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
  } else {
    quote! {
      drop(_guard);
    }
  };
  let body = &wrappee.block;
  let test_name = &wrappee.sig.ident;

  // Note that Rust does not allow us to have a test function with
  // #[should_panic] that has a non-unit return value.
  let ret = match &wrappee.sig.output {
    ReturnType::Default => quote! {},
    ReturnType::Type(_, type_) => quote! {-> #type_},
  };

  let logging_init = expand_logging_init();

  let result = quote! {
    #[#inner_test]
    #(#attrs)*
    #async_ fn #test_name() #ret {
      #async_ fn test_impl() #ret {
        #body
      }
      #logging_init
      let span = tracing::info_span!(stringify!(#test_name));
      #enter_
      let result = test_impl()#await_;
      if let Err(e) = &result {
        tracing::error!(error = ?e, "test failed");
      }
      #exit_
      if let Some(guard) = __guard { guard.teardown() } ;
      result
    }
  };

  result.into()
}
