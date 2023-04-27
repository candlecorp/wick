pub(crate) mod config;
mod expand_type;
use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
pub use config::configure;
use expand_type::expand_type;
use heck::{AsPascalCase, AsSnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use wick_config::config::{BoundInterface, OperationSignature};
use wick_config::{FetchOptions, WickConfiguration};
use wick_interface_types::{EnumSignature, EnumVariant, StructSignature, TypeDefinition};

use crate::module::Module;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
  In,
  Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Dependency {
  WasmRsRx,
  WasmRs,
  WasmRsCodec,
  WasmRsRuntime,
  SerdeJson,
  Bytes,
  WickComponent,
  AsyncTrait,
  WickPacket,
}

impl ToTokens for Dependency {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Dependency::WasmRsRx => {
        tokens.extend(
          quote! { pub use wick_component::wasmrs_rx; pub(crate) use wick_component::wasmrs_rx::{Observer,Observable}; },
        );
      }
      Dependency::WasmRsRuntime => {
        tokens.extend(quote! { pub use wick_component::runtime; });
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
        pub use wasmrs_guest::Value;
        #[cfg(not(target_family="wasm"))]
        pub use serde_json::Value;
      }),
      Dependency::Bytes => tokens.extend(quote! { pub use bytes::Bytes; }),
      Dependency::AsyncTrait => tokens.extend(quote! { pub use async_trait::async_trait; }),
      Dependency::WickComponent => tokens.extend(quote! { pub use wick_component; }),
    }
  }
}
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

fn get_typename_parts(name: &str) -> (Vec<&str>, &str) {
  let parts = name.split("::").collect::<Vec<_>>();
  let len = parts.len();
  let parts = parts.split_at(len - 1);
  (parts.0.to_vec(), parts.1[0])
}

fn _pathify_typename(name: &str) -> String {
  println!("name: {:?}", name);
  let (module_parts, item_part) = get_typename_parts(name);
  let mut path = module_parts
    .iter()
    .map(|p| format!("{}::", snake(p)))
    .collect::<String>();
  let name = pascal(item_part);

  path.push_str(&name);

  println!("structdef_name: {:?}", path);
  path
}

fn enumvariant_name(ty: &EnumVariant) -> String {
  pascal(&ty.name)
}

fn gen_register_channels<'a>(
  _config: &config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| {
    let name = Ident::new(&op_wrapper_name(op), Span::call_site());
    let string = &op.name;

    quote! {
      wasmrs_guest::register_request_channel("wick", #string, Box::new(#component::#name));
    }
  })
  .collect()
}

fn place_item(module: &Rc<RefCell<Module>>, mut path_parts_reverse: Vec<&str>, implementation: TokenStream) {
  if let Some(next) = path_parts_reverse.pop() {
    let module = module.borrow_mut().get_or_add(next);
    place_item(&module, path_parts_reverse, implementation);
  } else {
    module.borrow_mut().add(implementation);
  }
}

fn gen_types<'a>(config: &mut config::Config, ty: impl Iterator<Item = &'a TypeDefinition>) -> TokenStream {
  let types = ty.map(|v| gen_type(config, v)).collect::<Vec<_>>();
  let root = Module::new("types");
  for (mod_parts, implementation) in types {
    place_item(&root, mod_parts, implementation);
  }

  let borrowed = root.borrow();
  borrowed.codegen()
}

fn gen_provided(_config: &config::Config, required: &[BoundInterface]) -> TokenStream {
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

#[allow(clippy::needless_pass_by_value)]
fn wrap_parens(list: Vec<impl ToTokens>) -> TokenStream {
  if list.len() == 1 {
    quote! { #(#list),* }
  } else {
    quote! { (#(#list),*) }
  }
}

fn gen_response_streams(config: &mut config::Config, required: Vec<BoundInterface>) -> TokenStream {
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
            .map(|output| {
              (
                output.name.clone(),
                expand_type(config, Direction::In, false, &output.ty),
              )
            })
            .collect();
          let response_stream_types = response_streams.iter().map(|(_, ty)| quote! { WickStream<#ty>});
          let fan_out: Vec<_> = response_streams
            .iter()
            .map(|(n, t)| {
              quote! {
                (#n, #t)
              }
            })
            .collect();
          let types = wrap_parens(response_stream_types.collect());
          config.add_dep(Dependency::WickComponent);
          quote! {
            pub fn #name(&self, input: wick_packet::PacketStream) -> std::result::Result<#types,wick_packet::Error> {
              let mut stream = self.component.call(#op_name, input)?;
              Ok(wick_component::payload_fan_out!(stream, raw: false, [#(#fan_out),*]))
            }
          }
        })
        .collect::<Vec<_>>();

      config.add_dep(Dependency::WickPacket);
      quote! {
        pub struct #name {
          component: wick_packet::ComponentReference,
        }

        impl #name {
          pub fn new(component: wick_packet::ComponentReference) -> Self {
            Self { component }
          }
          pub fn component(&self) -> &wick_packet::ComponentReference {
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

fn gen_type<'a>(config: &mut config::Config, ty: &'a TypeDefinition) -> (Vec<&'a str>, TokenStream) {
  match ty {
    TypeDefinition::Enum(ty) => gen_enum(config, ty),
    TypeDefinition::Struct(ty) => gen_struct(config, ty),
  }
}

fn gen_enum<'a>(_config: &config::Config, ty: &'a EnumSignature) -> (Vec<&'a str>, TokenStream) {
  let (path_parts, item_part) = get_typename_parts(&ty.name);
  let name = Ident::new(item_part, Span::call_site());
  let variants = ty
    .variants
    .iter()
    .map(|v| {
      let name = Ident::new(&enumvariant_name(v), Span::call_site());
      quote! {#name}
    })
    .collect::<Vec<_>>();

  let from_index_arms = ty
    .variants
    .iter()
    .filter_map(|v| {
      let identname = Ident::new(&enumvariant_name(v), Span::call_site());
      v.index.map(|_| quote! {i => Self::#identname})
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
      let name = v
        .value
        .as_ref()
        .map_or_else(|| quote! {None}, |value| quote! { Some(#value) });
      quote! {Self::#identname => #name}
    })
    .collect::<Vec<_>>();

  let fromstr_match_arms = ty
    .variants
    .iter()
    .filter_map(|v| {
      let identname = Ident::new(&enumvariant_name(v), Span::call_site());
      v.value.as_ref().map(|name| quote! {#name => Ok(Self::#identname)})
    })
    .collect::<Vec<_>>();

  let enum_impl = quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    pub enum #name {
      #(#variants,)*
    }

    impl #name {
      #[allow(unused)]
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

fn gen_struct<'a>(config: &mut config::Config, ty: &'a StructSignature) -> (Vec<&'a str>, TokenStream) {
  let (module_parts, item_part) = get_typename_parts(&ty.name);
  let imported = ty.imported;

  let name = Ident::new(item_part, Span::call_site());
  let fields = ty
    .fields
    .iter()
    .map(|f| {
      let name = Ident::new(&snake(&f.name), Span::call_site());
      let ty = expand_type(config, Direction::In, imported, &f.ty);
      quote! {pub #name: #ty}
    })
    .collect::<Vec<_>>();

  let item = quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    pub struct #name {
      #(#fields,)*
    }
  };
  (module_parts, item)
}

fn gen_wrapper_fns<'a>(
  config: &mut config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| gen_wrapper_fn(config, component, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_wrapper_fn(config: &mut config::Config, component: &Ident, op: &OperationSignature) -> TokenStream {
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let wrapper_name = Ident::new(&op_wrapper_name(op), Span::call_site());
  let input_pairs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(config, Direction::In, false, &i.ty);
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
  config.add_dep(Dependency::WickPacket);
  config.add_dep(Dependency::WasmRs);
  config.add_dep(Dependency::WasmRsRuntime);

  quote! {
    fn #wrapper_name(mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>) -> std::result::Result<wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,Box<dyn std::error::Error + Send + Sync>> {
      let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
      let outputs = #outputs_name::new(channel.clone());

      runtime::spawn(async move {
        let #sanitized_input_names = wick_component::payload_fan_out!(input, #raw, [#(#input_pairs,)*]);
        if let Err(e) = #component::#impl_name(#(Box::pin(#inputs),)* outputs).await {
          let _ = channel.send_result(
            wick_packet::Packet::component_error(e.to_string()).into(),
          );
        }
      });

      Ok(Box::pin(rx))
    }
  }
}

fn gen_trait_fns<'a>(
  config: &mut config::Config,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| gen_trait_signature(config, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_trait_signature(config: &mut config::Config, op: &OperationSignature) -> TokenStream {
  let outputs_name = Ident::new(&op_outputs_name(op), Span::call_site());
  let trait_name = Ident::new(&format!("Op{}", &pascal(&op.name)), Span::call_site());
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let output_ports = op
    .outputs
    .iter()
    .map(|i| {
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(config, Direction::Out, false, &i.ty);
      quote! {pub(crate) #port_field_name: wick_packet::Output<#port_type>}
    })
    .collect::<Vec<_>>();
  let output_ports_new = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      quote! {#port_field_name: wick_packet::Output::new(#port_name, channel.clone())}
    })
    .collect::<Vec<_>>();

  let inputs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(config, Direction::In, false, &i.ty);
      quote! {#port_name: WickStream<#port_type>}
    })
    .collect::<Vec<_>>();

  let traits = gen_if(
    config.op_traits,
    || {
      config.add_dep(Dependency::AsyncTrait);
    },
    quote! {
      #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
      #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
      pub trait #trait_name {
        #[allow(unused)]
        async fn #impl_name(#(#inputs),*, outputs: #outputs_name) -> Result<()> {unimplemented!()}
      }
    },
  );

  let outputs = gen_if(
    config.output_structs,
    || {
      config.add_dep(Dependency::WickPacket);
      config.add_dep(Dependency::WasmRsRx);
      config.add_dep(Dependency::WasmRs);
    },
    quote! {
    pub struct #outputs_name {
      #[allow(unused)]
      #(#output_ports,)*
    }
    impl #outputs_name {
      pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
        Self {
          #(#output_ports_new,)*
        }
      }
    }},
  );

  quote! {
    #outputs
    #traits
  }
}

fn gen_component_impls<'a>(
  gen_config: &mut config::Config,
  component_name: &Ident,
  ops: impl Iterator<Item = &'a OperationSignature>,
  required: Vec<BoundInterface>,
) -> TokenStream {
  let provided = gen_if(!required.is_empty(), || {}, gen_provided(gen_config, &required));
  let response_streams = gen_response_streams(gen_config, required);
  let register_stmts = gen_register_channels(gen_config, component_name, ops);
  gen_config.add_dep(Dependency::WickPacket);
  gen_config.add_dep(Dependency::WasmRs);
  gen_config.add_dep(Dependency::WasmRsCodec);
  quote! {
    #[no_mangle]
    #[cfg(target_family = "wasm")]
    extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
      wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
      wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
      #(#register_stmts)*
    }

    #[cfg(target_family = "wasm")]
    thread_local! {
      static __CONFIG: std::cell::UnsafeCell<Option<SetupPayload>> = std::cell::UnsafeCell::new(None);
    }

    #[cfg(target_family = "wasm")]
    #[derive(Debug, serde::Deserialize)]
    pub(crate) struct SetupPayload {
      #[allow(unused)]
      pub(crate) provided: std::collections::HashMap<String,wick_packet::ComponentReference>
    }

    #[cfg(target_family = "wasm")]
    fn __setup(input: wasmrs_rx::BoxMono<wasmrs::Payload, wasmrs::PayloadError>) -> Result<wasmrs_rx::BoxMono<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
      Ok(wasmrs_rx::Mono::from_future(async move {
        match input.await {
          Ok(payload) => {
            let input = wasmrs_codec::messagepack::deserialize::<SetupPayload>(&payload.data).unwrap();
            __CONFIG.with(|cell| {
              #[allow(unsafe_code)]
              unsafe { &mut *cell.get() }.replace(input);
            });
            Ok(wasmrs::RawPayload::new_data(None, None))
          }
          Err(e) => {
            return Err(e);
          }
        }
      }).boxed())
    }

    #[allow(unused)]
    #[cfg(target_family = "wasm")]
    pub(crate) fn get_config() -> &'static SetupPayload {
      __CONFIG.with(|cell| {
        #[allow(unsafe_code)]
        unsafe { & *cell.get() }.as_ref().unwrap()
      })
    }
    #response_streams
    #provided

  }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn codegen(wick_config: WickConfiguration, gen_config: &mut config::Config) -> Result<String> {
  let (ops, types, required): (_, _, Vec<_>) = match &wick_config {
    wick_config::WickConfiguration::Component(config) => {
      let types = config.types()?;
      let requires = config.requires().values().cloned().collect();
      let ops = match config.component() {
        wick_config::config::ComponentImplementation::Wasm(c) => c.operations().clone(),
        wick_config::config::ComponentImplementation::Composite(c) => {
          c.operations().clone().into_iter().map(|(k, v)| (k, v.into())).collect()
        }
        _ => panic!("Code generation only supports `wick/component/wasm|composite` and `wick/types` configurations"),
      };
      (ops, types, requires)
    }
    wick_config::WickConfiguration::Types(config) => {
      (config.operations().clone(), config.types().to_vec(), Default::default())
    }
    _ => panic!("Code generation only supports `wick/component` and `wick/types` configurations"),
  };

  let component_name = Ident::new("Component", Span::call_site());
  let wrapper_fns = gen_wrapper_fns(gen_config, &component_name, ops.values());
  let trait_defs = gen_trait_fns(gen_config, ops.values());
  let typedefs = gen_types(gen_config, types.iter());

  let init = gen_if(
    !ops.is_empty(),
    || {},
    gen_component_impls(gen_config, &component_name, ops.values(), required),
  );
  let mut imports = Vec::new();
  for dep in gen_config.deps.iter() {
    imports.push(quote! {  #dep });
  }

  let imports = quote! {
      #( #imports )*
  };
  let components = gen_if(gen_config.components, || {}, {
    quote! {
      #[derive(Default, Clone)]
      pub struct #component_name;
      impl #component_name {
        #( #wrapper_fns )*
      }
    }
  });

  let expanded = quote! {
    #imports
    #[allow(unused)]
    pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
    pub use wick_component::anyhow::Result;

    #init

    #typedefs
    #( #trait_defs )*
    #components
  };
  Ok(expanded.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn gen_if(condition: bool, mut func: impl FnMut(), value: TokenStream) -> TokenStream {
  if condition {
    func();
    quote! { #value }
  } else {
    quote! {}
  }
}

pub fn build(config: config::Config) -> Result<()> {
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async_build(config))
}

pub async fn async_build(mut config: config::Config) -> Result<()> {
  let path = config.spec.as_path().to_str().unwrap();
  let wick_config = wick_config::WickConfiguration::fetch(path, FetchOptions::default()).await?;

  let src = codegen(wick_config, &mut config)?;
  tokio::fs::create_dir_all(&config.out_dir).await?;
  let target = config.out_dir.join("mod.rs");
  println!("Writing to {}", target.display());
  tokio::fs::write(target, src).await?;
  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_interface_types::TypeSignature;

  use super::*;
  use crate::generate::config::ConfigBuilder;
  use crate::Config;

  #[tokio::test]
  async fn test_build() -> Result<()> {
    let mut config = ConfigBuilder::new().spec("./tests/testdata/component.yaml").build()?;
    let wick_config = WickConfiguration::load_from_file(&config.spec).await.unwrap();

    let src = codegen(wick_config, &mut config)?;

    assert!(src.contains("pub struct Component"));

    Ok(())
  }

  #[test]
  fn test_expand_type() -> Result<()> {
    let mut config = Config::default();
    let ty = TypeSignature::Object;
    let src = expand_type(&mut config, Direction::In, false, &ty);

    assert_eq!(&src.to_string(), "Value");

    Ok(())
  }
}
