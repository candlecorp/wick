mod builder;
pub(crate) mod config;
use std::path::{Path, PathBuf};

use anyhow::Result;
pub use builder::configure;
use heck::{AsPascalCase, AsSnakeCase};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, Lit, LitStr};
use wick_config::config::{BoundInterface, FlowOperation, InterfaceDefinition, OperationSignature};
use wick_config::{normalize_path, WickConfiguration};
use wick_interface_types::{EnumSignature, EnumVariant, StructSignature, TypeDefinition};

fn snake(s: &str) -> String {
  AsSnakeCase(s).to_string()
}

fn pascal(s: &str) -> String {
  AsPascalCase(s).to_string()
}

fn op_wrapper_name(op: &OperationSignature) -> String {
  snake(&format!("{}_wrapper", op.name))
}

fn op_outputs_name(op: &OperationSignature) -> String {
  format!("Op{}Outputs", pascal(&op.name))
}

fn structdef_name(ty: &StructSignature) -> String {
  pascal(&ty.name)
}

fn enumdef_name(ty: &EnumSignature) -> String {
  pascal(&ty.name)
}

fn enumvariant_name(ty: &EnumVariant) -> String {
  pascal(&ty.name)
}

fn gen_register_channels<'a>(
  config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| {
    let name = Ident::new(&op_wrapper_name(op), Span::call_site());
    let string = &op.name;

    quote! {
        guest::register_request_channel("wick", #string, Box::new(#component::#name));
    }
  })
  .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
  In,
  Out,
}

fn expand_type(config: &config::Config, dir: Direction, ty: &wick_interface_types::TypeSignature) -> TokenStream {
  if config.raw && dir != Direction::Out {
    return quote! { wick_component::packet::Packet };
  }
  match ty {
    wick_interface_types::TypeSignature::Bool => quote! { bool },
    wick_interface_types::TypeSignature::U8 => quote! { u8 },
    wick_interface_types::TypeSignature::U16 => quote! { u16 },
    wick_interface_types::TypeSignature::U32 => quote! { u32 },
    wick_interface_types::TypeSignature::U64 => quote! { u64 },
    wick_interface_types::TypeSignature::I8 => quote! { i8 },
    wick_interface_types::TypeSignature::I16 => quote! { i16 },
    wick_interface_types::TypeSignature::I32 => quote! { i32 },
    wick_interface_types::TypeSignature::I64 => quote! { i64 },
    wick_interface_types::TypeSignature::F32 => quote! { f32 },
    wick_interface_types::TypeSignature::F64 => quote! { f64 },
    wick_interface_types::TypeSignature::String => quote! { String },
    wick_interface_types::TypeSignature::List { ty } => {
      let ty = expand_type(config, dir, ty);
      quote! { Vec<#ty> }
    }
    wick_interface_types::TypeSignature::Bytes => {
      quote! {bytes::Bytes}
    }
    wick_interface_types::TypeSignature::Custom(name) => {
      let ty = Ident::new(name, Span::call_site());
      quote! {#ty}
    }
    wick_interface_types::TypeSignature::Optional { ty } => {
      let ty = expand_type(config, dir, ty);
      quote! { Option<#ty> }
    }
    wick_interface_types::TypeSignature::Map { key, value } => {
      let key = expand_type(config, dir, key);
      let value = expand_type(config, dir, value);
      quote! { std::collections::HashMap<#key,#value> }
    }
    wick_interface_types::TypeSignature::Link { schemas } => quote! {wick_component::packet::ComponentReference},
    wick_interface_types::TypeSignature::Datetime => todo!("implement datetime in new codegen"),
    wick_interface_types::TypeSignature::Ref { reference } => todo!("implement ref in new codegen"),
    wick_interface_types::TypeSignature::Stream { ty } => {
      let ty = expand_type(config, dir, ty);
      quote! { WickStream<#ty> }
    }
    wick_interface_types::TypeSignature::Object => {
      quote! { wasmrs_guest::Value }
    }
    wick_interface_types::TypeSignature::AnonymousStruct(_) => todo!("implement anonymous struct in new codegen"),
  }
}

fn gen_types<'a>(config: &config::Config, ty: impl Iterator<Item = &'a TypeDefinition>) -> Vec<TokenStream> {
  ty.map(|v| gen_type(config, v))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_provided(config: &config::Config, required: &[BoundInterface]) -> TokenStream {
  if required.is_empty() {
    return quote! {};
  }

  let required_names = required
    .iter()
    .map(|r: &BoundInterface| {
      let name = Ident::new(&snake(&r.id), Span::call_site());
      let orig_name = &r.id;
      let response_name = Ident::new(&format!("{}Component", &pascal(&r.id)), Span::call_site());
      quote! { #name : #response_name::new(config.provided.get(#orig_name).cloned().unwrap()) }
    })
    .collect::<Vec<_>>();
  let fields = required
    .iter()
    .map(|v| {
      let name = Ident::new(&snake(&v.id), Span::call_site());
      let uc_name = Ident::new(&format!("{}Component", pascal(&v.id)), Span::call_site());
      quote! {pub #name: #uc_name}
    })
    .collect::<Vec<_>>();
  quote! {
    pub(crate) struct Provided {
      #(#fields),*
    }

    pub(crate) fn get_provided() -> Provided {
      let config = get_config();
      Provided {
        #(#required_names,)*
      }

    }
  }
}

fn gen_response_streams(config: &config::Config, required: Vec<BoundInterface>) -> TokenStream {
  let fields = required
    .into_iter()
    .map(|v| {
      let name = Ident::new(&format!("{}Component", &pascal(&v.id)), Span::call_site());
      let ops = v
        .kind
        .operations()
        .values()
        .map(|op| {
          let op_name = &op.name;
          let name = Ident::new(&snake(op_name), Span::call_site());
          let response_streams: Vec<_> = op
            .outputs
            .iter()
            .map(|output| (output.name.clone(), expand_type(config, Direction::In, &output.ty)))
            .collect();
          let response_stream_types = response_streams.iter().map(|(_, ty)| quote!{ WickStream<#ty>});
          let fan_out: Vec<_> = response_streams
            .iter()
            .map(|(n, t)| {
              quote! {
                (#n, #t)
              }
            })
            .collect();
          let types = if response_stream_types.len() == 1 {
            quote! { #(#response_stream_types),* }
          } else {
            quote! { (#(#response_stream_types),*) }
          };
          quote! {
            pub fn #name(&self, input: wick_component::packet::PacketStream) -> std::result::Result<#types,wick_component::packet::Error> {
              let mut stream = self.component.call(#op_name, input)?;
              Ok(wick_component::payload_fan_out!(stream, raw: false, [#(#fan_out),*]))
            }
          }
        })
        .collect::<Vec<_>>();

      quote! {
        pub struct #name {
          component: wick_component::packet::ComponentReference,
        }

        impl #name {
          pub fn new(component: wick_component::packet::ComponentReference) -> Self {
            Self { component }
          }
          pub fn component(&self) -> &wick_component::packet::ComponentReference {
            &self.component
          }
          #(#ops)*
        }
      }
    })
    .collect::<Vec<_>>();
  quote! {
      #(#fields),*

  }
}

fn gen_type(config: &config::Config, ty: &TypeDefinition) -> TokenStream {
  match ty {
    TypeDefinition::Enum(ty) => gen_enum(config, ty),
    TypeDefinition::Struct(ty) => gen_struct(config, ty),
  }
}

fn gen_enum(config: &config::Config, ty: &EnumSignature) -> TokenStream {
  let name = Ident::new(&enumdef_name(ty), Span::call_site());
  let variants = ty
    .variants
    .iter()
    .map(|v| {
      let name = Ident::new(&enumvariant_name(v), Span::call_site());
      quote! {#name}
    })
    .collect::<Vec<_>>();
  let display_match_arms = ty
    .variants
    .iter()
    .map(|v| {
      let identname = Ident::new(&enumvariant_name(v), Span::call_site());
      let name = v.name.clone();
      quote! {Self::#identname => f.write_str(#name)}
    })
    .collect::<Vec<_>>();
  let value_match_arms = ty
    .variants
    .iter()
    .map(|v| {
      let identname = Ident::new(&enumvariant_name(v), Span::call_site());
      let name = v.value.as_ref().map_or_else(|| quote! {None}, |v| quote! { Some(#v) });
      quote! {Self::#identname => #name}
    })
    .collect::<Vec<_>>();

  let fromstr_match_arms = ty
    .variants
    .iter()
    .map(|v| {
      let identname = Ident::new(&enumvariant_name(v), Span::call_site());
      let name = v.name.clone();
      quote! {#name => Ok(Self::#identname)}
    })
    .collect::<Vec<_>>();

  quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    pub enum #name {
      #(#variants,)*
    }

    impl #name {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        match self {
          #(#value_match_arms,)*
        }
      }
    }

    impl std::str::FromStr for #name {
      type Err = String;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          #(#fromstr_match_arms,)*
          _ => Err(s.to_owned())
        }
      }
    }

    impl std::fmt::Display for #name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          #(#display_match_arms,)*
        }
      }
    }
  }
}

fn gen_struct(config: &config::Config, ty: &StructSignature) -> TokenStream {
  let name = Ident::new(&structdef_name(ty), Span::call_site());
  let fields = ty
    .fields
    .iter()
    .map(|f| {
      let name = Ident::new(&snake(&f.name), Span::call_site());
      let ty = expand_type(config, Direction::In, &f.ty);
      quote! {pub #name: #ty}
    })
    .collect::<Vec<_>>();

  quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    pub struct #name {
      #(#fields,)*
    }
  }
}

fn gen_wrapper_fns<'a>(
  config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| gen_wrapper_fn(config, component, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_wrapper_fn(config: &config::Config, component: &Ident, op: &OperationSignature) -> TokenStream {
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let wrapper_name = Ident::new(&op_wrapper_name(op), Span::call_site());
  let string = &snake(&op.name);
  let input_pairs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(config, Direction::In, &i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect::<Vec<_>>();
  let inputs = op
    .inputs
    .iter()
    .map(|i| Ident::new(&snake(&i.name), Span::call_site()))
    .collect::<Vec<_>>();
  let outputs_name = Ident::new(&op_outputs_name(op), Span::call_site());
  let sanitized_input_names = if inputs.len() == 1 {
    quote! {#(#inputs)*}
  } else {
    quote! {(#(#inputs,)*)}
  };

  let raw = if config.raw {
    quote! {raw:true}
  } else {
    quote! {raw:false}
  };

  quote! {
    fn #wrapper_name(mut input: BoxFlux<Payload, PayloadError>) -> std::result::Result<BoxFlux<RawPayload, PayloadError>,Box<dyn std::error::Error + Send + Sync>> {
      let (channel, rx) = FluxChannel::<RawPayload, PayloadError>::new_parts();
      let outputs = #outputs_name::new(channel.clone());

      spawn(async move {
        let #sanitized_input_names = wick_component::payload_fan_out!(input, #raw, [#(#input_pairs,)*]);
        if let Err(e) = #component::#impl_name(#(Box::pin(#inputs),)* outputs).await {
          let _ = channel.send_result(
            wick_component::packet::Packet::component_error(e.to_string()).into(),
          );
        }
      });

      Ok(Box::pin(rx))
    }
  }
}

fn gen_trait_fns<'a>(
  config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| gen_trait_signature(config, component, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_trait_signature(config: &config::Config, component: &Ident, op: &OperationSignature) -> TokenStream {
  let outputs_name = Ident::new(&op_outputs_name(op), Span::call_site());
  let trait_name = Ident::new(&format!("Op{}", &pascal(&op.name)), Span::call_site());
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let input_pairs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(config, Direction::In, &i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect::<Vec<_>>();
  let output_ports = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(config, Direction::Out, &i.ty);
      quote! {pub(crate) #port_field_name: wick_component::packet::Output<#port_type>}
    })
    .collect::<Vec<_>>();
  let output_ports_new = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      quote! {#port_field_name: wick_component::packet::Output::new(#port_name, channel.clone())}
    })
    .collect::<Vec<_>>();

  let inputs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(config, Direction::In, &i.ty);
      quote! {#port_name: WickStream<#port_type>}
    })
    .collect::<Vec<_>>();

  quote! {
    pub struct #outputs_name {
      #[allow(unused)]
      #(#output_ports,)*
    }
    impl #outputs_name {
      pub fn new(channel: FluxChannel<RawPayload, PayloadError>) -> Self {
        Self {
          #(#output_ports_new,)*
        }
      }
    }
    #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
    pub trait #trait_name {
      #[allow(unused)]
      async fn #impl_name(#(#inputs),*, outputs: #outputs_name) -> Result<()> {unimplemented!()}
    }
  }
}

#[allow(clippy::needless_pass_by_value)]
fn codegen(wick_config: WickConfiguration, gen_config: &config::Config) -> Result<String> {
  let (ops, types, required): (_, _, Vec<_>) = match &wick_config {
    wick_config::WickConfiguration::Component(config) => match config.component() {
      wick_config::config::ComponentImplementation::Wasm(c) => (
        c.operations().clone(),
        c.types().to_vec(),
        c.requires().clone().into_values().collect(),
      ),
      wick_config::config::ComponentImplementation::Composite(c) => (
        c.operations().clone().into_iter().map(|(k, v)| (k, v.into())).collect(),
        c.types().to_vec(),
        c.requires().clone().into_values().collect(),
      ),
    },
    wick_config::WickConfiguration::Types(config) => (
      std::collections::HashMap::default(),
      config.types().to_vec(),
      Default::default(),
    ),
    _ => panic!("Code generation only supports `wick/component` and `wick/types` configurations"),
  };

  let component_name = Ident::new("Component", Span::call_site());
  let register_stmts = gen_register_channels(gen_config, &component_name, ops.values());
  let wrapper_fns = gen_wrapper_fns(gen_config, &component_name, ops.values());
  let trait_defs = gen_trait_fns(gen_config, &component_name, ops.values());
  let typedefs = gen_types(gen_config, types.iter());

  let init = if matches!(wick_config, WickConfiguration::Types(_)) {
    quote! {}
  } else {
    let provided = gen_provided(gen_config, &required);
    let response_streams = gen_response_streams(gen_config, required);
    quote! {
      #[no_mangle]
      extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
        guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
        guest::register_request_response("wick", "__setup", Box::new(__setup));
        #(#register_stmts)*
      }

      thread_local! {
        static __CONFIG: std::cell::UnsafeCell<Option<SetupPayload>> = std::cell::UnsafeCell::new(None);
      }

      #[derive(Debug, serde::Deserialize)]
      pub(crate) struct SetupPayload {
        #[allow(unused)]
        pub(crate) provided: std::collections::HashMap<String,wick_component::packet::ComponentReference>
      }

      fn __setup(input: BoxMono<Payload, PayloadError>) -> Result<BoxMono<RawPayload, PayloadError>, GenericError> {
        Ok(Mono::from_future(async move {
          match input.await {
            Ok(payload) => {
              let input = wasmrs_guest::deserialize::<SetupPayload>(&payload.data).unwrap();
              __CONFIG.with(|cell| {
                #[allow(unsafe_code)]
                unsafe { &mut *cell.get() }.replace(input);
              });
              Ok(RawPayload::new_data(None, None))
            }
            Err(e) => {
              return Err(e);
            }
          }
        }).boxed())
      }

      #[allow(unused)]
      pub(crate) fn get_config() -> &'static SetupPayload {
        __CONFIG.with(|cell| {
          #[allow(unsafe_code)]
          unsafe { & *cell.get() }.as_ref().unwrap()
        })
      }
      #response_streams
      #provided

    }
  };

  let expanded = quote! {
    #[allow(unused)]
    use guest::*;
    use wasmrs_guest as guest;
    #[allow(unused)]
    pub(crate) type WickStream<T> = BoxFlux<T, wick_component::anyhow::Error>;
    pub use wick_component::anyhow::Result;

    #init

    #( #typedefs )*
    #( #trait_defs )*

    #[derive(Default, Clone)]
    pub struct #component_name;
    impl #component_name {
      #( #wrapper_fns )*
    }
  };
  Ok(expanded.to_string())
}

#[allow(clippy::needless_pass_by_value)]
pub fn build(config: config::Config) -> Result<()> {
  let wick_yaml = std::fs::read_to_string(&config.spec)?;
  let wick_config = wick_config::WickConfiguration::from_yaml(&wick_yaml, &Some(config.spec.display().to_string()))?;

  let src = codegen(wick_config, &config)?;
  std::fs::create_dir_all(&config.out_dir)?;
  let target = config.out_dir.join("mod.rs");
  println!("Writing to {}", target.display());
  std::fs::write(target, src)?;
  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_interface_types::TypeSignature;

  use super::*;
  use crate::Config;

  // TODO: make better tests for the codegen. This one's pretty bad.
  #[tokio::test]
  async fn test_build() -> Result<()> {
    let config = Config::new().spec("./tests/testdata/component.yaml");
    let wick_config = WickConfiguration::load_from_file(&config.spec).await.unwrap();

    let src = codegen(wick_config, &config)?;

    assert!(src.contains("pub struct Component"));

    Ok(())
  }

  // TODO: make better tests for the codegen. This one's pretty bad.
  #[test]
  fn test_expand_type() -> Result<()> {
    let config = Config::default();
    let ty = TypeSignature::Object;
    let src = expand_type(&config, Direction::In, &ty);

    assert_eq!(&src.to_string(), "wasmrs_guest :: Value");

    Ok(())
  }
}
