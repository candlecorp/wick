use heck::{AsPascalCase, AsSnakeCase};
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use wick_config::config::{BoundInterface, OperationSignature};
use wick_interface_types::EnumVariant;

pub(crate) fn id(name: &str) -> Ident {
  Ident::new(name, Span::call_site())
}

pub(crate) fn component_id(r: &BoundInterface) -> String {
  format!("{}Component", &pascal(&r.id))
}

pub(crate) fn config_id(name: &str) -> String {
  format!("Op{}Config", pascal(name))
}

pub(crate) fn snake(s: &str) -> String {
  AsSnakeCase(s).to_string()
}

pub(crate) fn pascal(s: &str) -> String {
  AsPascalCase(s).to_string()
}

pub(crate) fn op_wrapper_name(op: &OperationSignature) -> String {
  snake(&format!("{}_wrapper", op.name()))
}

pub(crate) fn op_outputs_name(op: &OperationSignature) -> String {
  format!("Op{}Outputs", pascal(op.name()))
}

pub(crate) fn get_typename_parts(name: &str) -> (Vec<&str>, &str) {
  let parts = name.split("::").collect_vec();
  let len = parts.len();
  let parts = parts.split_at(len - 1);
  (parts.0.to_vec(), parts.1[0])
}

pub(crate) fn _pathify_typename(name: &str) -> String {
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

pub(crate) fn enumvariant_name(ty: &EnumVariant) -> String {
  pascal(&ty.name)
}
