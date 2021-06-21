
// This file is generated, do not edit
pub(crate) mod generated;
use vino_provider::VinoProviderComponent;

pub(crate) mod add;
pub(crate) mod bcrypt;
pub(crate) mod log;
pub(crate) mod string_to_bytes;

pub (crate) struct State{}

pub(crate) fn get_native_actor(
    name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = State> + Sync + Send>>{
  match name {

"add" => Some(Box::new(add::Component::default())),
"bcrypt" => Some(Box::new(bcrypt::Component::default())),
"log" => Some(Box::new(log::Component::default())),
"string-to-bytes" => Some(Box::new(string_to_bytes::Component::default())),
        _ => None,
    }
}
