#[macro_use]
extern crate log;

#[macro_use]
pub(crate) mod macros;

mod builder;
mod error;
mod host;
mod manifest;

pub type Result<T> = std::result::Result<T, VinoHostError>;
pub type Error = error::VinoHostError;

pub use builder::HostBuilder;
use error::VinoHostError;
pub use host::Host;

#[cfg(test)]
mod test {
    use crate::HostBuilder;
    use crate::Result;

    // #[test_env_log::test(actix_rt::test)]
    async fn nkeys() -> Result<()> {
        let kp = nkeys::KeyPair::new_server();
        println!("kp1: {:?}", kp);
        let pk1 = kp.public_key();
        let seed1 = kp.seed()?;
        println!("pk1: {}", pk1);
        println!("seed1: {}", seed1);

        let kp3 = nkeys::KeyPair::from_seed(&seed1)?;
        println!("kp3: {:?}", kp3);
        let pk3 = kp3.public_key();
        let seed3 = kp3.seed()?;
        println!("pk3: {}", pk3);
        println!("seed3: {}", seed3);

        let kp2 = nkeys::KeyPair::from_public_key(&pk1)?;
        println!("kp2: {:?}", kp2);
        let pk2 = kp2.public_key();
        let seed2 = kp2.seed()?;
        println!("pk2: {}", pk2);
        println!("seed2: {}", seed2);

        Ok(())
    }
}
