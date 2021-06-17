// This file is generated, do not edit
use crate::components::native_component_actor::{NativeActor, NativeCallback};
pub(crate) mod generated;

pub(crate) mod add;
pub(crate) mod bcrypt;
pub(crate) mod log;
pub(crate) mod string_to_bytes;

pub(crate) fn get_native_actor(name: &str) -> Option<Box<dyn NativeActor>> {
  match name {
    "vino::add" => Some(Box::new(add::Actor::default())),
    "vino::bcrypt" => Some(Box::new(bcrypt::Actor::default())),
    "vino::log" => Some(Box::new(log::Actor::default())),
    "vino::string-to-bytes" => Some(Box::new(string_to_bytes::Actor::default())),
    _ => None,
  }
}

pub(crate) fn new_native_actor(
  name: &str,
  callback: NativeCallback,
) -> Option<Box<dyn NativeActor>> {
  match name {
    "vino::add" => Some(Box::new(add::Actor::new(callback))),
    "vino::bcrypt" => Some(Box::new(bcrypt::Actor::new(callback))),
    "vino::log" => Some(Box::new(log::Actor::new(callback))),
    "vino::string-to-bytes" => Some(Box::new(string_to_bytes::Actor::new(callback))),
    _ => None,
  }
}
