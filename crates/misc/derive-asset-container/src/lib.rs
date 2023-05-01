use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Type};

#[proc_macro_derive(AssetManager, attributes(asset_managers, asset))]
pub fn derive_asset_container(input: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree.
  let ast = parse_macro_input!(input as DeriveInput);

  // Extract the name of the struct we're deriving the trait for.
  let name = &ast.ident;

  let asset_type: Option<Type> = ast
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("asset"))
    .map(|attribute| attribute.parse_args().expect("invalid attribute arguments"));

  match ast.data {
    syn::Data::Struct(ref data) => impl_struct(name, data, asset_type),
    syn::Data::Enum(ref data) => impl_enum(name, data, asset_type),
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

fn impl_struct(name: &Ident, data: &DataStruct, asset_type: Option<Type>) -> TokenStream {
  let fields = &data.fields;
  // Generate a list of field names as strings.
  let asset_fields = if let Some(asset_type) = asset_type.as_ref() {
    fields
      .iter()
      .filter(|field| &field.ty == asset_type)
      .filter(|field| !has_skip(&field.attrs))
      .map(|field| {
        if let Some(ident) = field.ident.as_ref() {
          ident.clone()
        } else {
          panic!("Unnamed fields are not supported.")
        }
      })
      .collect::<Vec<Ident>>()
  } else {
    Default::default()
  };

  // Generate a list of field names as strings.
  let inner_managers = fields
    .iter()
    .filter(|field| Some(&field.ty) != asset_type.as_ref())
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| {
      if let Some(ident) = field.ident.as_ref() {
        ident.clone()
      } else {
        panic!("Unnamed fields are not supported.")
      }
    })
    .collect::<Vec<Ident>>();

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &str) {
            #(self.#asset_fields.update_baseurl(baseurl);)*
            #(self.#inner_managers.set_baseurl(baseurl);)*
          }

          fn assets(&self) -> asset_container::Assets<#asset_type> {
            let mut assets = asset_container::Assets::default();
            #(assets.push(&self.#asset_fields);)*
            #(assets.extend(&mut self.#inner_managers.assets());)*
            assets
          }
      }
  };
  TokenStream::from(output)
}

fn impl_enum(name: &Ident, data: &DataEnum, asset_type: Option<Type>) -> TokenStream {
  let variants = &data.variants;

  let asset_variants = if let Some(asset_type) = asset_type.as_ref() {
    variants
      .iter()
      .filter(|v| v.fields.iter().any(|f| &f.ty == asset_type))
      .filter(|field| !has_skip(&field.attrs))
      .map(|field| field.ident.clone())
      .collect::<Vec<Ident>>()
  } else {
    Default::default()
  };

  let inner_managers = variants
    .iter()
    .filter(|v| v.fields.iter().any(|f| Some(&f.ty) != asset_type.as_ref()))
    .filter(|field| !has_skip(&field.attrs))
    .map(|field| field.ident.clone())
    .collect::<Vec<Ident>>();

  // Generate an implementation of the Assets trait for the struct.
  let output = quote! {
      impl asset_container::AssetManager for #name {
          type Asset = #asset_type;

          fn set_baseurl(&self, baseurl: &str) {
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
            let mut assets = asset_container::Assets::default();
            match self {
              #(Self::#asset_variants(v) => {
                assets.push(v);
              })*
              #(Self::#inner_managers(v) => {
                assets.extend(&mut asset_container::AssetManager::assets(v));
              })*
              _ => {}
            }
            assets
          }
      }
  };
  TokenStream::from(output)
}
