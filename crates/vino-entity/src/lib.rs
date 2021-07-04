pub mod entity;

pub mod error;
pub type Result<T> = std::result::Result<T, error::EntityError>;

pub use entity::{
  Entity,
  SystemEntity,
};
pub use error::EntityError as Error;

#[macro_use]
extern crate vino_macros;
