use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Dependency {
  WasmRsRx,
  WasmRs,
  WasmRsRuntime,
  SerdeJson,
  Bytes,
  WickComponent,
  Chrono,
  AsyncTrait,
  WickPacket,
}

// This has become largely useless once most dependencies have consolidated under wick_component.
// I'm unsure of whether to finish removing it or keep it around.
impl ToTokens for Dependency {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Dependency::WasmRsRx => {
        tokens.extend(quote! {});
      }
      Dependency::WasmRsRuntime => {
        tokens.extend(quote! {});
      }
      Dependency::Chrono => {
        tokens.extend(quote! {});
      }
      Dependency::WasmRs => {
        tokens.extend(quote! {});
      }
      Dependency::WickPacket => tokens.extend(quote! {}),
      Dependency::SerdeJson => tokens.extend(quote! {}),
      Dependency::Bytes => tokens.extend(quote! {}),
      Dependency::AsyncTrait => tokens.extend(quote! { pub use async_trait::async_trait; }),
      Dependency::WickComponent => tokens.extend(quote! {}),
    }
  }
}
