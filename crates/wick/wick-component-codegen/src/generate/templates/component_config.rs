use proc_macro2::TokenStream;
use quote::quote;
use wick_interface_types::{Field, StructDefinition, TypeDefinition};

use crate::generate::config;
use crate::generate::dependency::Dependency;
use crate::generate::ids::id;
use crate::generate::templates::TypeOptions;

pub(crate) fn component_config(config: &mut config::Config, fields: Option<Vec<Field>>) -> TokenStream {
  let config_name = "RootConfig";
  let config_id = id(config_name);

  let config_def = fields.map_or_else(
    || {
      quote! {
        #[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy, Default)]
        pub(crate) struct RootConfig {}
      }
    },
    |fields| {
      let (_, config_def) = super::type_def(
        config,
        &TypeDefinition::Struct(StructDefinition::new(config_name, fields)),
        TypeOptions::Defaults,
      );
      config_def
    },
  );

  config.add_dep(Dependency::WickPacket);
  config.add_dep(Dependency::WasmRsRx);
  config.add_dep(Dependency::WasmRs);

  quote! {
    #[cfg(target_family = "wasm")]
    thread_local! {
      static __CONFIG: std::cell::UnsafeCell<Option<SetupPayload>> = std::cell::UnsafeCell::new(None);
    }

    #config_def

    #[cfg(target_family = "wasm")]
    #[derive(Debug, serde::Deserialize)]
    pub(crate) struct SetupPayload {
      #[allow(unused)]
      pub(crate) provided: std::collections::HashMap<String,wick_packet::ComponentReference>,
      #[allow(unused)]
      pub(crate) config: #config_id,
    }

    #[cfg(target_family = "wasm")]
    fn __setup(input: wasmrs_rx::BoxMono<wasmrs::Payload, wasmrs::PayloadError>) -> Result<wasmrs_rx::BoxMono<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
      Ok(Box::pin(async move {
        let payload = input.await?;
        match wasmrs_codec::messagepack::deserialize::<SetupPayload>(&payload.data) {
          Ok(input) => {
            __CONFIG.with(|cell| {
              #[allow(unsafe_code)]
              unsafe { &mut *cell.get() }.replace(input);
            });
            Ok(wasmrs::RawPayload::new_data(None, None))
          }
          Err(e) => Err(wasmrs::PayloadError::application_error(e.to_string(), None)),
        }
      }))
    }

    #[allow(unused)]
    #[cfg(target_family = "wasm")]
    pub(crate) fn get_config() -> &'static SetupPayload {
      __CONFIG.with(|cell| {
        #[allow(unsafe_code)]
        unsafe { & *cell.get() }.as_ref().unwrap()
      })
    }

    #[allow(unused)]
    #[cfg(target_family = "wasm")]
    pub(crate) fn get_root_config() -> &'static #config_id {
      __CONFIG.with(|cell| {
        #[allow(unsafe_code)]
        &unsafe { & *cell.get() }.as_ref().unwrap().config
      })
    }
  }
}
