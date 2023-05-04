use asset_container::AssetFlags;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use structmeta::{NameArgs, StructMeta};
use syn::{parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Type};

#[derive(Debug, StructMeta)]
struct TypeOpts {
  lazy: bool,
  #[struct_meta(name = "asset")]
  ty: NameArgs<Type>,
}

#[proc_macro_derive(AssetManager, attributes(asset_managers, asset))]
pub fn derive_asset_container(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree.
  let ast = parse_macro_input!(input as DeriveInput);

  // Extract the name of the struct we're deriving the trait for.
  let name = &ast.ident;

  // Parse the attribute arguments.
  let opts = ast
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("asset"))
    .map(|attribute| attribute.parse_args::<TypeOpts>().expect("invalid attribute arguments"))
    .expect("no asset attribute");

  match ast.data {
    syn::Data::Struct(ref data) => impl_struct(name, data, opts),
    syn::Data::Enum(ref data) => impl_enum(name, data, opts),
    _ => panic!("Only structs and enums can derive the Assets trait."),
  }
}

fn has_skip(attr: &[Attribute]) -> bool {
  attr
    .iter()
    .find(|attr| attr.path().is_ident("asset"))
    .map(|attr| {
      let ident = attr.parse_args::<Ident>().expect("invalid attribute arguments");
      ident == "skip"
    })
    .unwrap_or(false)
}

fn impl_struct(name: &Ident, data: &DataStruct, opts: TypeOpts) -> TokenStream {
  let fields = &data.fields;
  // Generate a list of field names as strings.
  let asset_fields = fields
    .iter()
    .filter(|field| field.ty == opts.ty.args)
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| {
      if let Some(ident) = field.ident.as_ref() {
        ident.clone()
      } else {
        panic!("Unnamed fields are not supported.")
      }
    })
    .collect::<Vec<Ident>>();

  // Generate a list of field names as strings.
  let inner_managers = fields
    .iter()
    .filter(|field| field.ty != opts.ty.args)
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| {
      if let Some(ident) = field.ident.as_ref() {
        ident.clone()
      } else {
        panic!("Unnamed fields are not supported.")
      }
    })
    .collect::<Vec<Ident>>();
  let flags = if opts.lazy {
    AssetFlags::Lazy
  } else {
    AssetFlags::empty()
  }
  .bits();
  let flags = quote! {#flags};
  let asset_type = opts.ty.args;

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &std::path::Path) {
            #(self.#asset_fields.update_baseurl(baseurl);)*
            #(self.#inner_managers.set_baseurl(baseurl);)*
          }

          fn assets(&self) -> asset_container::Assets<#asset_type> {
            let mut assets = asset_container::Assets::new(vec![],self.get_asset_flags());
            #(assets.push(&self.#asset_fields);)*
            #(assets.extend(self.#inner_managers.assets());)*
            assets
          }

          fn get_asset_flags(&self) -> u32 {
            #flags
          }
      }
  };
  TokenStream::from(output)
}

fn impl_enum(name: &Ident, data: &DataEnum, opts: TypeOpts) -> TokenStream {
  let variants = &data.variants;

  let asset_variants = variants
    .iter()
    .filter(|v| v.fields.iter().any(|f| f.ty == opts.ty.args))
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| field.ident.clone())
    .collect::<Vec<Ident>>();

  let inner_managers = variants
    .iter()
    .filter(|v| v.fields.iter().any(|f| f.ty != opts.ty.args))
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| field.ident.clone())
    .collect::<Vec<Ident>>();

  let flags = if opts.lazy {
    AssetFlags::Lazy
  } else {
    AssetFlags::empty()
  }
  .bits();
  let flags = quote! {#flags};
  let asset_type = opts.ty.args;

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &std::path::Path) {
            match self {
              #(Self::#asset_variants(v) => {
                v.update_baseurl(baseurl);
              })*
              #(Self::#inner_managers(v) => {
                v.set_baseurl(baseurl);
              })*
              _ => {}
            }
          }

          fn assets(&self) -> asset_container::Assets<#asset_type> {
            let mut assets = asset_container::Assets::new(vec![],self.get_asset_flags());
            match self {
              #(Self::#asset_variants(v) => {
                assets.push(v);
              })*
              #(Self::#inner_managers(v) => {
                assets.extend(asset_container::AssetManager::assets(v));
              })*
              _ => {}
            }
            assets
          }

          fn get_asset_flags(&self) -> u32 {
            #flags
          }
      }
  };
  TokenStream::from(output)
}
