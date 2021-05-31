pub mod commands;
pub mod error;
mod utils;

use error::VinoCtlError;

pub type Result<T> = anyhow::Result<T, VinoCtlError>;
pub type Error = VinoCtlError;

#[macro_use]
extern crate log;

#[cfg(test)]
mod tests {

    #[actix_rt::test]
    async fn runs_crud_api_config() -> crate::Result<()> {
        Ok(())
    }
}
