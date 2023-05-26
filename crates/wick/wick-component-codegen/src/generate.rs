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
use wick_config::config::OperationSignature;
use wick_config::{FetchOptions, WickConfiguration};
use wick_interface_types::TypeDefinition;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
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
    let op_config = templates::op_config(config, op);
    let op_output = templates::op_outputs(config, op);
    let trait_sig = templates::trait_signature(config, op);
    quote! {
      pub mod #op_name {
        use super::*;
        #op_config
        #op_output
      }
      #trait_sig
    }
  })
  .collect_vec()
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn codegen(wick_config: WickConfiguration, gen_config: &mut config::Config) -> Result<String> {
  let (ops, types, required): (_, _, _) = match &wick_config {
    wick_config::WickConfiguration::Component(config) => {
      let types = config
        .types()?
        .into_iter()
        .sorted_by(|a, b| a.name().cmp(b.name()))
        .collect();
      let requires = config.requires().values().cloned().collect_vec();
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

  let component_name = id("Component");
  let wrapper_fns = gen_wrapper_fns(gen_config, &component_name, ops.values());
  let trait_defs = gen_trait_fns(gen_config, ops.values());
  let typedefs = gen_types("types", gen_config, types.iter());

  let init = f::gen_if(
    !ops.is_empty(),
    || {},
    templates::gen_component_impls(gen_config, &component_name, ops.values(), required),
  );

  let imports = gen_config.deps.iter().map(|dep| quote! { #dep }).collect_vec();
  let imports = quote! { #( #imports )* };

  let components = f::gen_if(gen_config.components, || {}, {
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
    pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, Box<dyn std::error::Error + Send + Sync>>;
    pub use wick_component::flow_component::Context;

    #init

    #typedefs
    #( #trait_defs )*
    #components
  };
  Ok(expanded.to_string())
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
  /****
   * See <project_root>/tests/codegen-tests/ for integration tests
   */
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
