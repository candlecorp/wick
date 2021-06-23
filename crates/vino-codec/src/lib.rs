pub mod error;
pub mod messagepack;

pub type Result<T> = std::result::Result<T, error::CodecError>;
pub type Error = error::CodecError;
