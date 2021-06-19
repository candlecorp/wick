pub mod cli;
pub mod error;

pub type Result<T> = std::result::Result<T, error::CliError>;
pub type Error = error::CliError;

pub use cli::init_basic;
