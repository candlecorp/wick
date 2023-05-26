use std::cell::RefCell;
use std::rc::Rc;

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

pub(crate) struct Module(String, Vec<TokenStream>, Vec<Rc<RefCell<Module>>>);

impl Module {
  pub(crate) fn new(name: &str) -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self(name.to_owned(), vec![], vec![])))
  }

  pub(crate) fn add(&mut self, implementation: TokenStream) {
    self.1.push(implementation);
  }

  pub(crate) fn add_module(&mut self, module: Rc<RefCell<Module>>) {
    self.2.push(module);
  }

  #[allow(clippy::option_if_let_else)]
  pub(crate) fn get_or_add(&mut self, name: &str) -> Rc<RefCell<Self>> {
    if let Some(module) = self.2.iter_mut().find(|m| m.borrow().0 == name) {
      module.clone()
    } else {
      self.add_module(Module::new(name));
      self.get_or_add(name)
    }
  }

  pub(crate) fn codegen(&self) -> TokenStream {
    let name = Ident::new(&self.0, Span::call_site());
    let implementations = &self.1;
    let modules = &self.2.iter().map(|m| m.borrow().codegen()).collect_vec();
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
    tokens.extend(self.codegen());
  }
}
