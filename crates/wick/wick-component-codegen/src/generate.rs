pub(crate) mod config;
mod dependency;
mod expand_type;
mod f;
mod ids;
mod module;
mod templates;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
pub use config::configure;
use expand_type::expand_type;
use ids::*;
use itertools::Itertools;
use module::Module;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use templates::TypeOptions;
use wick_config::{FetchOptions, WickConfiguration};
use wick_interface_types::{OperationSignature, OperationSignatures, TypeDefinition};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Direction {
  In,
  Out,
}

fn put_impl_in_path(module: &Rc<RefCell<Module>>, mut path_parts_reverse: Vec<&str>, implementation: TokenStream) {
  if let Some(next) = path_parts_reverse.pop() {
    let module = module.borrow_mut().get_or_add(next);
    put_impl_in_path(&module, path_parts_reverse, implementation);
  } else {
    module.borrow_mut().add(implementation);
  }
}

fn gen_types<'a>(
  module_name: &str,
  config: &mut config::Config,
  ty: impl Iterator<Item = &'a TypeDefinition>,
) -> TokenStream {
  let types = ty
    .map(|v| templates::type_def(config, v, TypeOptions::empty()))
    .collect_vec();
  let root = Module::new(module_name);
  for (mod_parts, implementation) in types {
    put_impl_in_path(&root, mod_parts, implementation);
  }

  let borrowed = root.borrow();
  borrowed.codegen()
}

fn gen_wrapper_fns<'a>(
  config: &mut config::Config,
  component: &Ident,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| templates::gen_wrapper_fn(config, component, op))
    .collect_vec()
}

fn gen_trait_fns<'a>(
  config: &mut config::Config,
  op: impl Iterator<Item = &'a OperationSignature>,
) -> Vec<TokenStream> {
  op.map(|op| {
    let op_name = id(&snake(op.name()));
    let op_config = templates::op_config(config, &generic_config_id(), op);
    let op_output = templates::op_outputs(config, op);
    let trait_sig = templates::trait_signature(config, op);
    let desc = format!("Types associated with the `{}` operation", op.name());
    quote! {
      #[doc = #desc]
      pub mod #op_name {
        #[allow(unused)]
        use super::*;
        #op_config
        #op_output
        #trait_sig
      }

    }
  })
  .collect_vec()
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn codegen(wick_config: WickConfiguration, gen_config: &mut config::Config) -> Result<String> {
  let (ops, types, required, root_config) = match &wick_config {
    wick_config::WickConfiguration::Component(comp) => {
      let types = comp
        .types()?
        .into_iter()
        .sorted_by(|a, b| a.name().cmp(b.name()))
        .collect();
      let root_config = comp.config().to_owned();
      let requires = comp.requires().clone().to_vec();
      let ops = match comp.component() {
        wick_config::config::ComponentImplementation::Wasm(c) => c.operation_signatures(),
        wick_config::config::ComponentImplementation::Composite(c) => c.operation_signatures(),
        _ => panic!("Code generation only supports `wick/component/wasm|composite` and `wick/types` configurations"),
      };
      (ops, types, requires, Some(root_config))
    }
    wick_config::WickConfiguration::Types(config) => (
      config.operation_signatures(),
      config.types().to_vec(),
      Default::default(),
      None,
    ),
    _ => panic!("Code generation only supports `wick/component` and `wick/types` configurations"),
  };

  let component_name = id("Component");
  let wrapper_fns = gen_wrapper_fns(gen_config, &component_name, ops.iter());
  let trait_defs = gen_trait_fns(gen_config, ops.iter());
  let typedefs = gen_types("types", gen_config, types.iter());

  let init = f::gen_if(
    !ops.is_empty(),
    || {},
    templates::gen_component_impls(gen_config, &component_name, ops.iter(), required),
  );

  let root_config = templates::component_config(gen_config, root_config);

  let imports = gen_config.deps.iter().map(|dep| quote! { #dep }).collect_vec();
  let imports = quote! { #( #imports )* };

  let components = f::gen_if(gen_config.components, || {}, {
    quote! {
      #[derive(Default, Clone)]
      #[doc = "The struct that the component implementation hinges around"]
      pub struct #component_name;
      impl #component_name {
        #( #wrapper_fns )*
      }
    }
  });

  let expanded = quote! {
    #imports

    #[allow(unused)]
    pub(crate) use wick_component::*;

    #[allow(unused)]
    pub(crate) use wick_component::WickStream;
    pub use wick_component::flow_component::Context;

    #init

    #root_config

    #[doc = "Additional generated types"]
    #typedefs
    #( #trait_defs )*
    #components
  };
  let reparsed = syn::parse_file(expanded.to_string().as_str())?;
  let formatted = prettyplease::unparse(&reparsed);
  Ok(formatted)
}

pub fn build(config: config::Config) -> Result<()> {
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async_build(config))
}

pub async fn async_build(mut config: config::Config) -> Result<()> {
  let path = config.spec.as_path().to_str().unwrap();
  let wick_config = wick_config::WickConfiguration::fetch(path, FetchOptions::default())
    .await?
    .into_inner();

  let src = codegen(wick_config, &mut config)?;
  tokio::fs::create_dir_all(&config.out_dir).await?;
  let target = config.out_dir.join("mod.rs");
  println!("Writing to {}", target.display());
  tokio::fs::write(target, src).await?;
  Ok(())
}

#[cfg(test)]
mod test {
  /****
   * See <project_root>/tests/codegen-tests/ for integration tests
   */
  use anyhow::Result;
  use wick_interface_types::Type;

  use super::*;
  use crate::generate::config::ConfigBuilder;
  use crate::Config;

  #[tokio::test]
  async fn test_build() -> Result<()> {
    let mut config = ConfigBuilder::new().spec("./tests/testdata/component.yaml").build()?;
    let wick_config = WickConfiguration::fetch(&config.spec, Default::default())
      .await
      .unwrap()
      .finish()?;

    let src = codegen(wick_config, &mut config)?;

    assert!(src.contains("pub struct Component"));

    Ok(())
  }

  #[test]
  fn test_expand_type() -> Result<()> {
    let mut config = Config::default();
    let ty = Type::Object;
    let src = expand_type(&mut config, Direction::In, false, &ty);

    assert_eq!(&src.to_string(), "wick_component :: Value");

    Ok(())
  }
}
