
// This file is generated, do not edit
use crate::native_component_actor::NativeActor;
pub(crate) mod generated;

pub(crate) mod add;
pub(crate) mod bcrypt;
pub(crate) mod log;
pub(crate) mod string_to_bytes;

pub(crate) fn get_native_actor(name:String) -> Option<Box<dyn NativeActor>> {
    match name.as_str() {

"vino::add" => Some(Box::new(add::Actor {})),
"vino::bcrypt" => Some(Box::new(bcrypt::Actor {})),
"vino::log" => Some(Box::new(log::Actor {})),
"vino::string-to-bytes" => Some(Box::new(string_to_bytes::Actor {})),
        _ => None,
    }
}
