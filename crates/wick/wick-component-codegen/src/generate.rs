mod builder;
pub(crate) mod config;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
pub use builder::configure;
use heck::{AsPascalCase, AsSnakeCase};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, Lit, LitStr};
use wick_config::config::{BoundInterface, FlowOperation, InterfaceDefinition, OperationSignature};
use wick_config::{normalize_path, FetchOptions, WickConfiguration};
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

fn get_typename_parts(name: &str) -> (Vec<&str>, &str) {
  let parts = name.split("::").collect::<Vec<_>>();
  let len = parts.len();
  let parts = parts.split_at(len - 1);
  (parts.0.to_vec(), parts.1[0])
}

fn pathify_typename(name: &str) -> String {
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

fn structdef_path(ty: &StructSignature) -> String {
  pathify_typename(&ty.name)
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

fn expand_type(
  config: &config::Config,
  dir: Direction,
  imported: bool,
  ty: &wick_interface_types::TypeSignature,
) -> TokenStream {
  println!("expand_type:  {:?}", ty);
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
      let ty = expand_type(config, dir, imported, ty);
      quote! { Vec<#ty> }
    }
    wick_interface_types::TypeSignature::Bytes => {
      quote! {bytes::Bytes}
    }
    wick_interface_types::TypeSignature::Custom(name) => {
      let (mod_parts, item_part) = get_typename_parts(name);
      let mod_parts = mod_parts.iter().map(|p| Ident::new(p, Span::call_site()));
      let ty = Ident::new(item_part, Span::call_site());
      let location = if imported {
        quote! {}
      } else {
        quote! {types::}
      };
      quote! {#location #(#mod_parts ::)*#ty}
    }
    wick_interface_types::TypeSignature::Optional { ty } => {
      let ty = expand_type(config, dir, imported, ty);
      quote! { Option<#ty> }
    }
    wick_interface_types::TypeSignature::Map { key, value } => {
      let key = expand_type(config, dir, imported, key);
      let value = expand_type(config, dir, imported, value);
      quote! { std::collections::HashMap<#key,#value> }
    }
    wick_interface_types::TypeSignature::Link { schemas } => quote! {wick_component::packet::ComponentReference},
    wick_interface_types::TypeSignature::Datetime => todo!("implement datetime in new codegen"),
    wick_interface_types::TypeSignature::Ref { reference } => todo!("implement ref in new codegen"),
    wick_interface_types::TypeSignature::Stream { ty } => {
      let ty = expand_type(config, dir, imported, ty);
      quote! { WickStream<#ty> }
    }
    wick_interface_types::TypeSignature::Object => {
      quote! { wasmrs_guest::Value }
    }
    wick_interface_types::TypeSignature::AnonymousStruct(_) => todo!("implement anonymous struct in new codegen"),
  }
}

struct Module(String, Vec<TokenStream>, Vec<Rc<RefCell<Module>>>);

impl Module {
  fn new(name: &str) -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self(name.to_owned(), vec![], vec![])))
  }

  fn add(&mut self, implementation: TokenStream) {
    self.1.push(implementation);
  }

  fn add_module(&mut self, module: Rc<RefCell<Module>>) {
    self.2.push(module);
  }

  #[allow(clippy::option_if_let_else)]
  fn get_or_add(&mut self, name: &str) -> Rc<RefCell<Self>> {
    if let Some(module) = self.2.iter_mut().find(|m| m.borrow().0 == name) {
      module.clone()
    } else {
      self.add_module(Module::new(name));
      self.get_or_add(name)
    }
  }

  fn to_tokens_(&self) -> TokenStream {
    let name = Ident::new(&self.0, Span::call_site());
    let implementations = &self.1;
    let modules = &self.2.iter().map(|m| m.borrow().to_tokens_()).collect::<Vec<_>>();
    quote! {
      pub mod #name {
        #[allow(unused)]
        use super::#name;
        #(#implementations)*
        #(#modules)*
      }
    }
  }
}

impl ToTokens for Module {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_tokens_());
  }
}

fn place_item(module: &Rc<RefCell<Module>>, mut path_parts_reverse: Vec<&str>, implementation: TokenStream) {
  if let Some(next) = path_parts_reverse.pop() {
    let module = module.borrow_mut().get_or_add(next);
    place_item(&module, path_parts_reverse, implementation);
  } else {
    module.borrow_mut().add(implementation);
  }
}

fn gen_types<'a>(config: &config::Config, ty: impl Iterator<Item = &'a TypeDefinition>) -> TokenStream {
  let types = ty.map(|v| gen_type(config, v)).collect::<Vec<_>>();
  let mut mod_map: HashMap<String, Vec<&TokenStream>> = HashMap::default();
  let mut root = Module::new("types");
  for (mod_parts, implementation) in types {
    place_item(&root, mod_parts, implementation);
  }

  let borrowed = root.borrow();
  borrowed.to_tokens_()
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
            .map(|output| (output.name.clone(), expand_type(config, Direction::In, false,&output.ty)))
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

fn gen_type<'a>(config: &config::Config, ty: &'a TypeDefinition) -> (Vec<&'a str>, TokenStream) {
  match ty {
    TypeDefinition::Enum(ty) => gen_enum(config, ty),
    TypeDefinition::Struct(ty) => gen_struct(config, ty),
  }
}

fn gen_enum<'a>(config: &config::Config, ty: &'a EnumSignature) -> (Vec<&'a str>, TokenStream) {
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

  let enum_impl = quote! {
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
  };
  (path_parts, enum_impl)
}

fn gen_struct<'a>(config: &config::Config, ty: &'a StructSignature) -> (Vec<&'a str>, TokenStream) {
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
      let port_type = expand_type(config, Direction::In, false, &i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect::<Vec<_>>();
  let output_ports = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(config, Direction::Out, false, &i.ty);
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
      let port_type = expand_type(config, Direction::In, false, &i.ty);
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

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn codegen(wick_config: WickConfiguration, gen_config: &config::Config) -> Result<String> {
  let (ops, types, required): (_, _, Vec<_>) = match &wick_config {
    wick_config::WickConfiguration::Component(config) => {
      let types = config.types()?;
      match config.component() {
        wick_config::config::ComponentImplementation::Wasm(c) => (
          c.operations().clone(),
          types,
          c.requires().clone().into_values().collect(),
        ),
        wick_config::config::ComponentImplementation::Composite(c) => (
          c.operations().clone().into_iter().map(|(k, v)| (k, v.into())).collect(),
          types,
          c.requires().clone().into_values().collect(),
        ),
      }
    }
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
      #[cfg(target_family = "wasm")]
      extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
        guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
        guest::register_request_response("wick", "__setup", Box::new(__setup));
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
        pub(crate) provided: std::collections::HashMap<String,wick_component::packet::ComponentReference>
      }

      #[cfg(target_family = "wasm")]
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
  };

  let guest = if matches!(wick_config, WickConfiguration::Types(_)) {
    quote! {}
  } else {
    quote! {
      #[allow(unused)]
      use guest::*;
      use wasmrs_guest as guest;
    }
  };
  let expanded = quote! {
    #guest
    #[allow(unused)]
    pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
    pub use wick_component::anyhow::Result;

    #init

    #typedefs
    #( #trait_defs )*

    #[derive(Default, Clone)]
    pub struct #component_name;
    impl #component_name {
      #( #wrapper_fns )*
    }
  };
  Ok(expanded.to_string())
}

pub fn build(config: config::Config) -> Result<()> {
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async_build(config))
}

pub async fn async_build(config: config::Config) -> Result<()> {
  let path = config.spec.as_path().to_str().unwrap();
  let wick_config = wick_config::WickConfiguration::fetch(path, FetchOptions::default()).await?;

  let src = codegen(wick_config, &config)?;
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
  use crate::Config;

  #[tokio::test]
  async fn test_build() -> Result<()> {
    let config = Config::new().spec("./tests/testdata/component.yaml");
    let wick_config = WickConfiguration::load_from_file(&config.spec).await.unwrap();

    let src = codegen(wick_config, &config)?;

    assert!(src.contains("pub struct Component"));

    Ok(())
  }

  #[test]
  fn test_expand_type() -> Result<()> {
    let config = Config::default();
    let ty = TypeSignature::Object;
    let src = expand_type(&config, Direction::In, false, &ty);

    assert_eq!(&src.to_string(), "wasmrs_guest :: Value");

    Ok(())
  }
}
