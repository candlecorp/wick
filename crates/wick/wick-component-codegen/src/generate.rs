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
use wick_config::FlowOperation;
use wick_interface_types::{EnumSignature, StructSignature, TypeDefinition};

fn snake(s: &str) -> String {
  AsSnakeCase(s).to_string()
}

fn pascal(s: &str) -> String {
  AsPascalCase(s).to_string()
}

fn op_wrapper_name(op: &FlowOperation) -> String {
  snake(&format!("{}_wrapper", op.name))
}

fn op_outputs_name(op: &FlowOperation) -> String {
  format!("Op{}Outputs", pascal(&op.name))
}

fn structdef_name(ty: &StructSignature) -> String {
  pascal(&ty.name)
}

fn enumdef_name(ty: &EnumSignature) -> String {
  pascal(&ty.name)
}

fn gen_register_channels<'a>(component: &Ident, op: impl Iterator<Item = &'a FlowOperation>) -> Vec<TokenStream> {
  op.map(|op| {
    let name = Ident::new(&op_wrapper_name(op), Span::call_site());
    let string = &op.name;

    quote! {
        guest::register_request_channel("wick", #string, #component::#name);
    }
  })
  .collect()
}

fn expand_type(ty: &wick_interface_types::TypeSignature) -> TokenStream {
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
      let ty = expand_type(ty);
      quote! { Vec<#ty> }
    }
    wick_interface_types::TypeSignature::Bytes => quote! {bytes::Bytes},
    wick_interface_types::TypeSignature::Custom(name) => {
      let ty = Ident::new(name, Span::call_site());
      quote! {#ty}
    }
    wick_interface_types::TypeSignature::Optional { ty } => {
      let ty = expand_type(ty);
      quote! { Option<#ty> }
    }
    wick_interface_types::TypeSignature::Map { key, value } => {
      let key = expand_type(key);
      let value = expand_type(value);
      quote! { HashMap<#key,#value> }
    }
    wick_interface_types::TypeSignature::Link { schemas } => quote! {wick_component::packet::CollectionLink},
    wick_interface_types::TypeSignature::Datetime => todo!(),
    wick_interface_types::TypeSignature::Value => todo!(),
    wick_interface_types::TypeSignature::Internal(_) => todo!(),
    wick_interface_types::TypeSignature::Ref { reference } => todo!(),
    wick_interface_types::TypeSignature::Stream { ty } => todo!(),
    wick_interface_types::TypeSignature::Struct => todo!(),
    wick_interface_types::TypeSignature::AnonymousStruct(_) => todo!(),
  }
}

fn gen_types<'a>(ty: impl Iterator<Item = &'a TypeDefinition>) -> Vec<TokenStream> {
  ty.map(gen_type).collect::<Vec<_>>().into_iter().collect()
}

fn gen_type(ty: &TypeDefinition) -> TokenStream {
  match ty {
    TypeDefinition::Enum(ty) => gen_enum(ty),
    TypeDefinition::Struct(ty) => gen_struct(ty),
  }
}

fn gen_enum(ty: &EnumSignature) -> TokenStream {
  let name = Ident::new(&enumdef_name(ty), Span::call_site());
  let variants = ty
    .variants
    .iter()
    .map(|v| {
      let name = Ident::new(&snake(&v.name), Span::call_site());
      quote! {#name}
    })
    .collect::<Vec<_>>();

  quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum #name {
      #(#variants,)*
    }
  }
}

fn gen_struct(ty: &StructSignature) -> TokenStream {
  let name = Ident::new(&structdef_name(ty), Span::call_site());
  let fields = ty
    .fields
    .iter()
    .map(|f| {
      let name = Ident::new(&snake(&f.name), Span::call_site());
      let ty = expand_type(&f.ty);
      quote! {pub #name: #ty}
    })
    .collect::<Vec<_>>();

  quote! {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct #name {
      #(#fields,)*
    }
  }
}

fn gen_wrapper_fns<'a>(component: &Ident, op: impl Iterator<Item = &'a FlowOperation>) -> Vec<TokenStream> {
  op.map(|op| gen_wrapper_fn(component, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_wrapper_fn(component: &Ident, op: &FlowOperation) -> TokenStream {
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let wrapper_name = Ident::new(&op_wrapper_name(op), Span::call_site());
  let string = &snake(&op.name);
  let input_pairs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(&i.ty);
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

  quote! {
    fn #wrapper_name(mut input: FluxReceiver<Payload, PayloadError>) -> std::result::Result<FluxReceiver<RawPayload, PayloadError>,Box<dyn std::error::Error + Send + Sync>> {
      let (channel, rx) = FluxChannel::<RawPayload, PayloadError>::new_parts();
      let outputs = #outputs_name::new(channel);

      spawn(async move {
        let #sanitized_input_names = payload_fan_out!(input, [#(#input_pairs,)*]);
        if let Err(e) = #component::#impl_name(#(#inputs,)* outputs).await {
          panic!("{}: {}", #string, e);
        }
      });

      Ok(rx)
    }
  }
}

fn gen_trait_fns<'a>(component: &Ident, op: impl Iterator<Item = &'a FlowOperation>) -> Vec<TokenStream> {
  op.map(|op| gen_trait_signature(component, op))
    .collect::<Vec<_>>()
    .into_iter()
    .collect()
}

fn gen_trait_signature(component: &Ident, op: &FlowOperation) -> TokenStream {
  let outputs_name = Ident::new(&op_outputs_name(op), Span::call_site());
  let trait_name = Ident::new(&format!("Op{}", &pascal(&op.name)), Span::call_site());
  let impl_name = Ident::new(&snake(&op.name), Span::call_site());
  let input_pairs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_type = expand_type(&i.ty);
      quote! {(#port_name, #port_type)}
    })
    .collect::<Vec<_>>();
  let output_ports = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(&i.ty);
      quote! {#port_field_name: Output<#port_type>}
    })
    .collect::<Vec<_>>();
  let output_ports_new = op
    .outputs
    .iter()
    .map(|i| {
      let port_name = &i.name;
      let port_field_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(&i.ty);
      quote! {#port_field_name: Output::new(#port_name, channel)}
    })
    .collect::<Vec<_>>();

  let inputs = op
    .inputs
    .iter()
    .map(|i| {
      let port_name = Ident::new(&snake(&i.name), Span::call_site());
      let port_type = expand_type(&i.ty);
      quote! {#port_name: WickStream<#port_type>}
    })
    .collect::<Vec<_>>();

  quote! {
    pub struct #outputs_name {
      pub(crate) #(#output_ports,)*
    }
    impl #outputs_name {
      pub fn new(channel: FluxChannel<RawPayload, PayloadError>) -> Self {
        Self {
          #(#output_ports_new,)*
        }
      }
    }
    #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
    pub trait #trait_name {
      #[allow(unused)]
      async fn #impl_name(#(#inputs),*, outputs: #outputs_name) -> Result<()> {unimplemented!()}
    }
  }
}

fn codegen(config: &config::Config) -> Result<String> {
  let component = wick_config::ComponentConfiguration::load_from_file(&config.spec).unwrap();
  let component_name = Ident::new(
    pascal(&component.name().clone().unwrap_or("component".to_owned())).as_ref(),
    Span::call_site(),
  );
  let register_stmts = gen_register_channels(&component_name, component.operations().values());
  let wrapper_fns = gen_wrapper_fns(&component_name, component.operations().values());
  let trait_defs = gen_trait_fns(&component_name, component.operations().values());
  let typedefs = gen_types(component.types().iter());

  let expanded = quote! {
    use guest::*;
    use wasmrs_guest as guest;
    use wick_component::payload_fan_out;
    use wick_component::packet::{Packet};
    pub(crate) type WickStream<T> = FluxReceiver<T, wick_component::anyhow::Error>;
    pub use wick_component::anyhow::Result;

    pub(crate) struct Output<T> where T: serde::Serialize{
      channel: FluxChannel<RawPayload, PayloadError>,
      name: String,
      _phantom: std::marker::PhantomData<T>
    }
    impl<T> Output<T>  where T: serde::Serialize{
      pub fn new(name: impl AsRef<str>, channel: FluxChannel<RawPayload, PayloadError>) -> Self {
        Self {
          channel,
          name: name.as_ref().to_owned(),
          _phantom:Default::default()
        }
      }
      pub fn send(&mut self, value: T) {
        let _ = self.channel.send_result(Packet::encode(&self.name, value).into());
      }
      pub fn done(&mut self) {
        let _ = self.channel.send_result(Packet::done(&self.name).into());
      }
      pub fn error(&mut self, err: impl AsRef<str>) {
        let _ = self.channel.send_result(Packet::err(&self.name, err).into());
      }
    }

    #[no_mangle]
    extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
      guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
      #(#register_stmts)*
    }

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
  let src = codegen(&config)?;
  std::fs::create_dir_all(&config.out_dir)?;
  let target = config.out_dir.join("mod.rs");
  println!("Writing to {}", target.display());
  std::fs::write(target, src)?;
  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::Config;

  #[test]
  fn test_build() -> Result<()> {
    let config = Config::new().spec("./tests/testdata/component.yaml");
    let src = codegen(&config)?;

    assert!(src.contains("pub struct TestComponent"));

    Ok(())
  }
}
