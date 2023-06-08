use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Dependency {
  WasmRsRx,
  WasmRs,
  WasmRsCodec,
  WasmRsRuntime,
  SerdeJson,
  Bytes,
  WickComponent,
  Chrono,
  AsyncTrait,
  WickPacket,
}

impl ToTokens for Dependency {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Dependency::WasmRsRx => {
        tokens.extend(quote! {
        pub use wick_component::wasmrs_rx;
        #[allow(unused)]
        pub(crate) use wick_component::wasmrs_rx::{Observer,Observable}; });
      }
      Dependency::WasmRsRuntime => {
        tokens.extend(quote! { pub use wick_component::runtime; });
      }
      Dependency::Chrono => {
        tokens.extend(quote! { pub use wick_component::datetime; });
      }

      Dependency::WasmRs => {
        tokens.extend(quote! { pub use wick_component::wasmrs; });
      }
      Dependency::WasmRsCodec => {
        tokens.extend(quote! { pub use wick_component::wasmrs_codec; });
      }
      Dependency::WickPacket => tokens.extend(quote! { pub use wick_component::packet as wick_packet; }),
      Dependency::SerdeJson => tokens.extend(quote! {
        #[cfg(target_family="wasm")]
        pub use wick_component::wasmrs_guest::Value;
        #[cfg(not(target_family="wasm"))]
        pub use serde_json::Value;
      }),
      Dependency::Bytes => tokens.extend(quote! { pub use wick_component::bytes::Bytes; }),
      Dependency::AsyncTrait => tokens.extend(quote! { pub use async_trait::async_trait; }),
      Dependency::WickComponent => tokens.extend(quote! {
        pub use wick_component;
        pub use wick_component::StreamExt;
      }),
    }
  }
}
